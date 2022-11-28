#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::prelude::*;

enum Format {
    Text,
    Binary
}

enum TxMode {
    OnRequest,
    OnEvent,
    Continuous
}

const DATA_FRAME_START:  u8 = 0b10000000;


struct Context {
    format: Format,
    mode: TxMode,
    sample_request_pending: bool,
    state: ControllerState,
}

#[derive(Default, PartialEq)]
struct ControllerState {
    xaxis: u16,
    yaxis: u16,
    buttons: u8,
}

#[derive(Clone)]
enum AxisReadState {
    ReadingX,
    ReadingY {x: u16},
}

impl Context {
    fn new() -> Context {
        Context{
            format: Format::Text, 
            mode: TxMode::OnEvent,
            sample_request_pending: false,
            state: ControllerState::default(),
        }
    }
    fn set_format(&mut self, format: Format) {
        self.format = format
    }
    fn set_txmode(&mut self, txmode: TxMode) {
        self.mode = txmode;
    }
    fn request_sample(&mut self) {
        self.sample_request_pending = true;
    }

    fn write_state_text<W>(&self, w: &mut W) -> Result<(), <W as ufmt::uWrite>::Error>
    where W: ufmt::uWrite
    {
        let st = &(self.state);
        return ufmt::uwriteln!(w, "x:{}, y:{}, b:{}", st.xaxis, st.yaxis, st.buttons);
    }

    fn write_state_binary(&self, w: &mut [u8; 4]) {
        let st = &(self.state);

        // start data frame. Only the start byte has bit 7 set
        // (FYI we count bits from 0..7, LSB to MSB)
        // We encode the buttons here too.
        w[0] = DATA_FRAME_START | (st.buttons);

        // x/y axis encoding: both axes contain 10 bits of data,
        // in total 20 bits. Our data frame bytes can hold 7
        // bits of data, so in 3 bytes we could 21 bits.
        // 
        // We encode the 7 most significant bits of x and y
        // in separate bytes, so if that level of accuracy is 
        // sufficient, these values can be used as they are.
        // The remaining 3 bits of each axis are incoded into 
        // the following byte, the ones for x in the high nibble
        // (bits 4..7) and the ones for y in the low nibble 
        // (bits 0..3)

        let x_7highbits = ((st.xaxis >> 3) & 0x7f) as u8;
        let y_7highbits = ((st.xaxis >> 3) & 0x7f) as u8;
        let xy_3lowbits = ((st.xaxis << 4) & 0x70) as u8 | ((st.yaxis << 0) & 0x07) as u8;
        w[1] = x_7highbits;
        w[2] = y_7highbits;
        w[3] = xy_3lowbits;
    }

    fn write<W>(&self, w: &mut W)
    where W: ufmt::uWrite + embedded_hal::serial::Write<u8>
    {
        match self.format {
            Format::Text => {self.write_state_text(w);}
            Format::Binary => {
                let mut buf=[0u8;4];
                self.write_state_binary(&mut buf);
                for b in buf {
                    nb::block!(w.write(b));
                }
            },
        };
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // assign IO pins D2..D5 as buttons 0..3
    let buttons = (
        pins.d2.into_pull_up_input(),
        pins.d3.into_pull_up_input(),
        pins.d4.into_pull_up_input(),
        pins.d5.into_pull_up_input(),
    );
    
    // ADC will read buttons for X and Y axes
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    
    let axes = (
        pins.a0.into_analog_input(&mut adc),
        pins.a1.into_analog_input(&mut adc)
    );

    let (mut r, mut w) = serial.split();

    ufmt::uwriteln!(&mut w, "Hello from Arduino!\r").void_unwrap();

    let mut ctx = Context::new();
    let mut axis_read_state = AxisReadState::ReadingX;


    loop {
        // Read a byte from the serial connection
        if let Ok(input) = r.read() {
            match input {
                // format commands
                b'b' => ctx.set_format(Format::Binary),
                b't' => ctx.set_format(Format::Text),
                // tx mode and request commands
                b'r' => ctx.set_txmode(TxMode::OnRequest),
                b'e' => ctx.set_txmode(TxMode::OnEvent),
                b'c' => ctx.set_txmode(TxMode::Continuous),
                // request a sample command
                b's' => ctx.request_sample(),
                // all others are ignored
                _ => ()
            }
        }

        // read buttons
        let mut state = ControllerState::default();
        state.buttons = 0
            |((buttons.0.is_low() as u8) << 0)
            |((buttons.1.is_low() as u8) << 1)
            |((buttons.2.is_low() as u8) << 2)
            |((buttons.3.is_low() as u8) << 3);
        state.xaxis = ctx.state.xaxis;
        state.yaxis = ctx.state.yaxis;

        match axis_read_state.clone() {
            AxisReadState::ReadingX => if let Ok(x) = adc.read_nonblocking(&axes.0) {
                axis_read_state = AxisReadState::ReadingY { x }
            },
            AxisReadState::ReadingY { x } => if let Ok(y) = adc.read_nonblocking(&axes.1) {
                state.xaxis = x;
                state.yaxis = y;
                axis_read_state = AxisReadState::ReadingX;
            }
        }
        
        match ctx.mode {
            TxMode::OnEvent => if state != ctx.state {
                ctx.state = state;
                
                ctx.write(&mut w);
            },
            TxMode::Continuous => {
                ctx.state = state;
                ctx.write(&mut w);
            },
            TxMode::OnRequest => if ctx.sample_request_pending {
                ctx.sample_request_pending = false;
                ctx.state = state;
                ctx.write(&mut w);
            },
        }
    }
}
