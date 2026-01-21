use idevice::{
    services::springboardservices::SpringBoardServicesClient, usbmuxd::UsbmuxdAddr, IdeviceService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Looking for connected devices...");

    let addr = UsbmuxdAddr::default();
    let mut usbmuxd = addr.connect(1).await?;
    let devices = usbmuxd.get_devices().await?;

    if devices.is_empty() {
        println!("❌ No devices found!");
        return Ok(());
    }

    let device = &devices[0];
    println!("✅ Found device: {}", device.udid);

    let provider = device.to_provider(addr, "icon_state_test");

    println!("✅ Connecting to SpringBoard services...");
    let mut client = SpringBoardServicesClient::connect(&provider).await?;
    println!("✅ Connected!\n");

    println!("📱 Getting current icon state...");
    let mut icon_state = client.get_icon_state(None).await?;

    println!("✅ Successfully retrieved icon state");

    println!("\n🔄 Attempting to swap last two icons on the first page...");

    let mut swapped = false;
    if let plist::Value::Array(icon_lists) = &mut icon_state {
        println!("✓ Icon state is an array with {} pages", icon_lists.len());

        if let Some(plist::Value::Array(first_page_sections)) = icon_lists.first_mut() {
            println!("✓ First page has {} sections", first_page_sections.len());

            if let Some(plist::Value::Array(first_section)) = first_page_sections.first_mut() {
                let len = first_section.len();
                println!("✓ First section has {} items", len);

                let mut icon_count = 0;
                for item in first_section.iter() {
                    if let plist::Value::Dictionary(_) = item {
                        icon_count += 1;
                    }
                }
                println!("✓ Found {} real icons (excluding placeholders)", icon_count);

                if icon_count >= 2 {
                    let mut last_two_indices = vec![];
                    for (i, item) in first_section.iter().enumerate().rev() {
                        if let plist::Value::Dictionary(dict) = item {
                            if dict.contains_key("displayIdentifier") {
                                last_two_indices.push(i);
                                if last_two_indices.len() == 2 {
                                    break;
                                }
                            }
                        }
                    }

                    if last_two_indices.len() == 2 {
                        let idx1 = last_two_indices[1];
                        let idx2 = last_two_indices[0];

                        if let Some(plist::Value::Dictionary(dict1)) = first_section.get(idx1) {
                            if let Some(name1) =
                                dict1.get("displayName").and_then(|n| n.as_string())
                            {
                                println!("  Icon at position {}: {}", idx1, name1);
                            }
                        }
                        if let Some(plist::Value::Dictionary(dict2)) = first_section.get(idx2) {
                            if let Some(name2) =
                                dict2.get("displayName").and_then(|n| n.as_string())
                            {
                                println!("  Icon at position {}: {}", idx2, name2);
                            }
                        }

                        println!("  Swapping positions {} and {}", idx1, idx2);
                        first_section.swap(idx1, idx2);
                        println!("  ✅ Swap completed");
                        swapped = true;
                    }
                } else {
                    println!(
                        "  ⚠️  Not enough icons (need at least 2, found {})",
                        icon_count
                    );
                }
            }
        }
    }

    if swapped {
        println!("\n📱 Setting modified icon state back to device...");
        client.set_icon_state(icon_state).await?;
        println!("✅ Successfully set icon state!");

        println!("\n🎉 Check your device - the last two icons should be swapped!");
    } else {
        println!("\n⚠️  Did not modify icon state");
    }

    Ok(())
}
