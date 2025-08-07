use std::process::Command;
use std::str;

/// Get battery percentage for Logitech MX Keys Mini via Solaar
///
/// Returns:
/// - Ok(percentage) if battery level found (0-100)
/// - Err(String) with error description if failed
///
/// Example:
/// ```
/// match get_mx_keys_mini_battery() {
///     Ok(percentage) => println!("Battery: {}%", percentage),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn get_mx_keys_mini_battery() -> Result<u8, String> {
    get_battery_by_vendor_product_id("046d", "B369")
}

/// Get battery percentage for any Logitech device by vendor and product ID
///
/// # Arguments
/// * `vendor_id` - Vendor ID in hex format (e.g., "046d" for Logitech)
/// * `product_id` - Product ID in hex format (e.g., "B369" for MX Keys Mini)
///
/// # Returns
/// * `Ok(u8)` - Battery percentage (0-100)
/// * `Err(String)` - Error description
pub fn get_battery_by_vendor_product_id(vendor_id: &str, product_id: &str) -> Result<u8, String> {
    // Execute solaar show command
    let output = Command::new("solaar")
        .arg("show")
        .output()
        .map_err(|e| format!("Failed to execute solaar: {}", e))?;

    if !output.status.success() {
        return Err("Solaar command failed".to_string());
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in solaar output: {}", e))?;

    parse_battery_from_solaar_output(stdout, vendor_id, product_id)
}

/// Parse battery percentage from solaar output by vendor/product ID
fn parse_battery_from_solaar_output(output: &str, vendor_id: &str, product_id: &str) -> Result<u8, String> {
    let mut in_device = false;
    let mut found_device = false;
    let mut battery_percentage: Option<u8> = None;

    // Normalize IDs to uppercase for comparison
    let target_vendor = vendor_id.to_uppercase();
    let target_product = product_id.to_uppercase();

    for line in output.lines() {
        let trimmed_line = line.trim();

        // Start of device section (non-indented line, not version/receiver info)
        if !line.starts_with(' ') &&
            !line.starts_with('\t') &&
            !line.starts_with("solaar version") &&
            !line.starts_with("Приёмник") &&
            !line.contains("Device path") {

            // If we had a previous matching device with battery, return it
            if found_device && battery_percentage.is_some() {
                return Ok(battery_percentage.unwrap());
            }

            // Reset for new device
            in_device = true;
            found_device = false;
            battery_percentage = None;
            continue;
        }

        // Look for USB id line in current device
        if in_device && trimmed_line.contains("USB id") {
            // Parse "USB id       : 046d:B369" format
            if let Some(colon_pos) = trimmed_line.find(':') {
                let after_colon = &trimmed_line[colon_pos + 1..].trim();
                if let Some((device_vendor, device_product)) = after_colon.split_once(':') {
                    let device_vendor = device_vendor.trim().to_uppercase();
                    let device_product = device_product.trim().to_uppercase();

                    if device_vendor == target_vendor && device_product == target_product {
                        found_device = true;
                    }
                }
            }
        }

        // Look for battery info in the matched device
        if in_device && found_device && trimmed_line.contains("Battery:") {
            // Parse "Battery: 95%, 0." format
            if let Some(battery_start) = trimmed_line.find("Battery:") {
                let battery_part = &trimmed_line[battery_start + 8..]; // Skip "Battery:"

                // Extract percentage using regex-like parsing
                for part in battery_part.split_whitespace() {
                    if part.ends_with('%') || part.ends_with("%,") {
                        let num_part = part.trim_end_matches('%').trim_end_matches("%,");
                        if let Ok(percentage) = num_part.parse::<u8>() {
                            if percentage <= 100 {
                                battery_percentage = Some(percentage);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if we found the device and battery at end of input
    if found_device && battery_percentage.is_some() {
        Ok(battery_percentage.unwrap())
    } else if found_device {
        Err("Device found but no battery information available".to_string())
    } else {
        Err(format!("Device with vendor ID {} and product ID {} not found", vendor_id, product_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mx_keys_mini() {
        let sample_output = r#"solaar version 1.1.14

MX Keys Mini
     Device path  : /dev/hidraw5
     USB id       : 046d:B369
     Codename     : MX Keys Mini
     Kind         : keyboard
     Protocol     : HID++ 4.5
     Battery: 95%, 0.
"#;

        let result = parse_battery_from_solaar_output(sample_output, "046d", "B369");
        assert_eq!(result, Ok(95));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let sample_output = r#"solaar version 1.1.14

MX Keys Mini
     USB id       : 046D:b369
     Battery: 42%, 0.
"#;

        let result = parse_battery_from_solaar_output(sample_output, "046d", "B369");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_device_not_found() {
        let sample_output = r#"solaar version 1.1.14

Some Other Device
     USB id       : 046d:C094
     Battery: 50%, 0.
"#;

        let result = parse_battery_from_solaar_output(sample_output, "046d", "B369");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_multiple_devices() {
        let sample_output = r#"solaar version 1.1.14

PRO X Wireless
     USB id       : 046d:C094
     Battery: 25%, 0.

MX Keys Mini
     USB id       : 046d:B369
     Battery: 87%, 0.
"#;

        let result = parse_battery_from_solaar_output(sample_output, "046d", "B369");
        assert_eq!(result, Ok(87));
    }
}

// Example usage function
fn test_solaar_adapter(){
    match get_mx_keys_mini_battery() {
        Ok(percentage) => println!("MX Keys Mini Battery: {}%", percentage),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Alternative usage with custom vendor/product IDs
    match get_battery_by_vendor_product_id("046d", "B369") {
        Ok(percentage) => println!("Device Battery: {}%", percentage),
        Err(e) => eprintln!("Error: {}", e),
    }
}