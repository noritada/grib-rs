pub(crate) fn jma_amedas_temperature(degrees_celsius: f32) -> [u8; 4] {
    match degrees_celsius {
        v if v > degrees_celsius_to_kelvin(35.) => [180, 0, 104, 255],
        v if v > degrees_celsius_to_kelvin(30.) => [255, 40, 0, 255],
        v if v > degrees_celsius_to_kelvin(25.) => [255, 153, 0, 255],
        v if v > degrees_celsius_to_kelvin(20.) => [250, 245, 0, 255],
        v if v > degrees_celsius_to_kelvin(15.) => [255, 255, 150, 255],
        v if v > degrees_celsius_to_kelvin(10.) => [255, 255, 240, 255],
        v if v > degrees_celsius_to_kelvin(5.) => [185, 235, 255, 255],
        v if v > degrees_celsius_to_kelvin(0.) => [0, 150, 255, 255],
        v if v > degrees_celsius_to_kelvin(-5.) => [0, 65, 255, 255],
        _ => [0, 32, 128, 255],
    }
}

fn degrees_celsius_to_kelvin(value: f32) -> f32 {
    value + 273.15
}
