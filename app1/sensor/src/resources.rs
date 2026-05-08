use esp_hal::assign_resources;

assign_resources! {
    pub Resources<'d> {
        bt: BluetoothResources<'d> {
            bt:  BT,
        },
        dht11: DHT11Resources<'d> {
            pin:  GPIO16,
        },
        dps310: DPS310Resources<'d> {
            i2c: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
        light: LightResources<'d> {
            adc:  ADC1,
            pin: GPIO34,
        },
        rain: RainResources<'d> {
            pin:  GPIO23,
        },
        wind: WindResources<'d> {
            adc:  ADC2,
            direction:  GPIO35,
            speed:  GPIO27,
        },
    }
}
