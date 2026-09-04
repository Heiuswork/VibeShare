use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::ImageEncoder;
use qrcode::QrCode;

pub fn png_data_url(text: &str) -> Result<String, String> {
    let code = QrCode::new(text.as_bytes()).map_err(|error| error.to_string())?;
    let image = code.render::<image::Luma<u8>>().min_dimensions(512, 512).build();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::L8,
        )
        .map_err(|error| error.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}
