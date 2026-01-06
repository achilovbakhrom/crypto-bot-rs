// Utility functions for the bot module
// QR code generation, formatting helpers, etc.

use qrcode::QrCode;
use image::Luma;

pub fn generate_qr_code(data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let code = QrCode::new(data)?;
    let image = code.render::<Luma<u8>>().build();

    let mut buffer = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;

    Ok(buffer)
}
