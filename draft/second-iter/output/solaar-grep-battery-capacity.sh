#!/bin/bash

# Parse Solaar output by vendor_id and product_id
get_battery_by_vendor_product_id() {
    local vendor_id="$1"  # e.g., "046d" for Logitech
    local product_id="$2" # e.g., "B369" for MX Keys Mini

    # Validate input
    if [[ -z "$vendor_id" || -z "$product_id" ]]; then
        echo "Usage: get_battery_by_vendor_product_id <vendor_id> <product_id>"
        echo "Example: get_battery_by_vendor_product_id 046d B369"
        return 1
    fi

    # Check if solaar is available
    if ! command -v solaar &> /dev/null; then
        echo "Error: solaar not found" >&2
        return 1
    fi

    local result
    result=$(solaar show 2>/dev/null | awk -v vid="$vendor_id" -v pid="$product_id" '
    BEGIN {
        in_device = 0
        found_device = 0
        battery = ""
        # Convert to uppercase for comparison
        vid = toupper(vid)
        pid = toupper(pid)
    }

    # Start of device section (non-indented line that is not version/receiver info)
    /^[^ ]/ && !/^solaar version/ && !/^Приёмник/ && !/^Device path.*:/ {
        in_device = 1
        found_device = 0
        battery = ""
        next
    }

    # Look for USB id line in current device
    in_device && /USB id/ {
        # Extract vendor:product from "USB id : 046d:B369" format
        match($0, /USB id.*: *([0-9A-Fa-f]+):([0-9A-Fa-f]+)/, arr)
        if (arr[1] != "" && arr[2] != "") {
            device_vid = toupper(arr[1])
            device_pid = toupper(arr[2])
            if (device_vid == vid && device_pid == pid) {
                found_device = 1
            }
        }
    }

    # Look for battery info in the matched device
    in_device && found_device && /Battery:/ {
        match($0, /Battery: *([0-9]+)%/, arr)
        if (arr[1] != "") {
            battery = arr[1]
        }
    }

    # End of device section (empty line or new device)
    /^$/ || (/^[^ ]/ && !/^solaar version/ && !/^Приёмник/ && !/^Device path.*:/) {
        if (found_device && battery != "") {
            print battery
            exit
        }
        in_device = 0
        found_device = 0
        battery = ""
    }

    # Handle end of input
    END {
        if (found_device && battery != "") {
            print battery
        }
    }')

    if [[ -n "$result" && "$result" =~ ^[0-9]+$ ]] && [ "$result" -ge 0 ] && [ "$result" -le 100 ]; then
        echo "$result"
        return 0
    else
        return 1
    fi
}

# Alternative method using Model ID parsing
get_battery_by_model_id() {
    local model_id="$1"  # e.g., "B36900000000" for MX Keys Mini

    if [[ -z "$model_id" ]]; then
        echo "Usage: get_battery_by_model_id <model_id>"
        echo "Example: get_battery_by_model_id B36900000000"
        return 1
    fi

    if ! command -v solaar &> /dev/null; then
        echo "Error: solaar not found" >&2
        return 1
    fi

    local result
    result=$(solaar show 2>/dev/null | awk -v mid="$model_id" '
    BEGIN {
        in_device = 0
        found_device = 0
        battery = ""
        mid = toupper(mid)
    }

    /^[^ ]/ && !/^solaar version/ && !/^Приёмник/ && !/^Device path.*:/ {
        in_device = 1
        found_device = 0
        battery = ""
        next
    }

    # Look for Model ID line
    in_device && /Model ID/ {
        match($0, /Model ID: *([0-9A-Fa-f]+)/, arr)
        if (arr[1] != "" && toupper(arr[1]) == mid) {
            found_device = 1
        }
    }

    in_device && found_device && /Battery:/ {
        match($0, /Battery: *([0-9]+)%/, arr)
        if (arr[1] != "") {
            battery = arr[1]
        }
    }

    /^$/ || (/^[^ ]/ && !/^solaar version/ && !/^Приёмник/ && !/^Device path.*:/) {
        if (found_device && battery != "") {
            print battery
            exit
        }
        in_device = 0
        found_device = 0
        battery = ""
    }

    END {
        if (found_device && battery != "") {
            print battery
        }
    }')

    if [[ -n "$result" && "$result" =~ ^[0-9]+$ ]] && [ "$result" -ge 0 ] && [ "$result" -le 100 ]; then
        echo "$result"
        return 0
    else
        return 1
    fi
}

# Generic function that tries multiple ID methods
get_battery_by_ids() {
    local vendor_id="$1"
    local product_id="$2"
    local model_id="$3"

    local battery

    # Try vendor_id + product_id first
    if [[ -n "$vendor_id" && -n "$product_id" ]]; then
        if battery=$(get_battery_by_vendor_product_id "$vendor_id" "$product_id"); then
            echo "$battery"
            return 0
        fi
    fi

    # Try model_id as fallback
    if [[ -n "$model_id" ]]; then
        if battery=$(get_battery_by_model_id "$model_id"); then
            echo "$battery"
            return 0
        fi
    fi

    return 1
}

# Predefined function for your specific MX Keys Mini
get_mx_keys_mini_battery() {
    get_battery_by_ids "046d" "B369" "B36900000000"
}

# Function to list all devices with their IDs (for discovery)
list_devices_with_ids() {
    solaar show 2>/dev/null | awk '
    BEGIN { in_device = 0; device_name = "" }

    /^[^ ]/ && !/^solaar version/ && !/^Приёмник/ && !/^Device path.*:/ {
        if (in_device && device_name != "") {
            printf "\n"
        }
        in_device = 1
        device_name = $0
        printf "Device: %s\n", device_name
        next
    }

    in_device && /USB id/ {
        match($0, /USB id.*: *([0-9A-Fa-f]+:[0-9A-Fa-f]+)/, arr)
        if (arr[1] != "") printf "  USB ID: %s\n", arr[1]
    }

    in_device && /Model ID/ {
        match($0, /Model ID: *([0-9A-Fa-f]+)/, arr)
        if (arr[1] != "") printf "  Model ID: %s\n", arr[1]
    }

    in_device && /Battery:/ {
        match($0, /Battery: *([0-9]+)%/, arr)
        if (arr[1] != "") printf "  Battery: %s%%\n", arr[1]
    }'
}

# Main execution
main() {
    echo "=== Parsing by IDs Test ==="

    echo "Listing all devices with IDs:"
    list_devices_with_ids
    echo ""

    echo "Testing MX Keys Mini by vendor/product ID (046d:B369):"
    if battery=$(get_battery_by_vendor_product_id "046d" "B369"); then
        echo "  Battery: ${battery}%"
    else
        echo "  Failed"
    fi

    echo "Testing MX Keys Mini by Model ID (B36900000000):"
    if battery=$(get_battery_by_model_id "B36900000000"); then
        echo "  Battery: ${battery}%"
    else
        echo "  Failed"
    fi

    echo "Testing combined method:"
    if battery=$(get_mx_keys_mini_battery); then
        echo "  Battery: ${battery}%"
    else
        echo "  Failed"
    fi

    echo ""
    echo "=== Usage Examples ==="
    echo "# For MX Keys Mini specifically:"
    echo "battery=\$(get_mx_keys_mini_battery) && echo \"Battery: \${battery}%\""
    echo ""
    echo "# For any device by vendor:product ID:"
    echo "battery=\$(get_battery_by_vendor_product_id \"046d\" \"B369\") && echo \"Battery: \${battery}%\""
    echo ""
    echo "# For any device by model ID:"
    echo "battery=\$(get_battery_by_model_id \"B36900000000\") && echo \"Battery: \${battery}%\""
}

# Execute main if script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main
fi