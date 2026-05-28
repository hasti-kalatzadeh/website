#![no_std]
#![no_main]

use embassy_executor::Spawner;

use embassy_stm32::gpio::{
    Input,
    Level,
    Output,
    Pull,
    Speed,
};

use embassy_stm32::i2c::{
    I2c,
    Master,
};

use embassy_stm32::mode::Blocking;

use embassy_time::{
    Duration,
    Timer,
};

use {defmt_rtt as _, panic_probe as _};

const LCD_ADDR: u8 = 0x27;

// ================= LCD =================

fn lcd_send_nibble(
    i2c: &mut I2c<'_, Blocking, Master>,
    nibble: u8,
    rs: u8,
) {

    let data = nibble | rs | 0x08;

    let _ = i2c.blocking_write(
        LCD_ADDR,
        &[data | 0x04],
    );

    let _ = i2c.blocking_write(
        LCD_ADDR,
        &[data],
    );

}

fn lcd_send_byte(
    i2c: &mut I2c<'_, Blocking, Master>,
    byte: u8,
    rs: u8,
) {

    lcd_send_nibble(
        i2c,
        byte & 0xF0,
        rs,
    );

    lcd_send_nibble(
        i2c,
        (byte << 4) & 0xF0,
        rs,
    );

}

fn lcd_cmd(
    i2c: &mut I2c<'_, Blocking, Master>,
    cmd: u8,
) {

    lcd_send_byte(
        i2c,
        cmd,
        0x00,
    );

}

fn lcd_data(
    i2c: &mut I2c<'_, Blocking, Master>,
    data: u8,
) {

    lcd_send_byte(
        i2c,
        data,
        0x01,
    );

}

fn lcd_print(
    i2c: &mut I2c<'_, Blocking, Master>,
    text: &str,
) {

    for b in text.bytes() {

        lcd_data(
            i2c,
            b,
        );

    }

}

// ================= MAIN =================

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let p =
        embassy_stm32::init(
            Default::default()
        );

    // BUZZER

    let mut buzzer =
        Output::new(
            p.PA1,
            Level::Low,
            Speed::Low,
        );

    // BLUE BUTTON

    let button =
        Input::new(
            p.PC13,
            Pull::None,
        );

    // LCD

    let mut i2c =
        I2c::new_blocking(
            p.I2C1,
            p.PB6,
            p.PB7,
            Default::default(),
        );

    Timer::after(
        Duration::from_millis(100)
    ).await;

    // LCD INIT

    lcd_cmd(&mut i2c, 0x33);

    lcd_cmd(&mut i2c, 0x32);

    lcd_cmd(&mut i2c, 0x28);

    lcd_cmd(&mut i2c, 0x0C);

    lcd_cmd(&mut i2c, 0x06);

    lcd_cmd(&mut i2c, 0x01);

    Timer::after(
        Duration::from_millis(20)
    ).await;

    // START IN DANGER MODE

    let mut safe_mode = false;

    loop {

        // BLUE BUTTON -> SAFE

        if button.is_high() {

            Timer::after(
                Duration::from_millis(300)
            ).await;

            if button.is_high() {

                safe_mode = true;

            }

        }

        // CLEAR LCD

        lcd_cmd(
            &mut i2c,
            0x01,
        );

        Timer::after(
            Duration::from_millis(20)
        ).await;

        // SAFE MODE

        if safe_mode {

            lcd_cmd(
                &mut i2c,
                0x80,
            );

            lcd_print(
                &mut i2c,
                "     SAFE      ",
            );

            buzzer.set_low();

            Timer::after(
                Duration::from_secs(1)
            ).await;

        }

        // DANGER MODE

        else {

            lcd_cmd(
                &mut i2c,
                0x80,
            );

            lcd_print(
                &mut i2c,
                "    DANGER     ",
            );

            // BEEP BEEP

            buzzer.set_high();

            Timer::after(
                Duration::from_millis(100)
            ).await;

            buzzer.set_low();

            Timer::after(
                Duration::from_millis(300)
            ).await;

        }

    }

}