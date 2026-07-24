import struct
import zlib
import json

def create_test_png(path="test_image.png", width=256, height=256):
    raw_rows = []
    for _ in range(height):
        row = b"\x00"
        pixel = bytes([220, 220, 220, 255])
        row += pixel * width
        raw_rows.append(row)
    raw_data = b"".join(raw_rows)

    def make_chunk(chunk_type, data):
        length = struct.pack(">I", len(data))
        crc = zlib.crc32(chunk_type + data) & 0xFFFFFFFF
        return length + chunk_type + data + struct.pack(">I", crc)

    png_signature = b"\x89PNG\r\n\x1a\n"
    ihdr_data = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    ihdr = make_chunk(b"IHDR", ihdr_data)
    compressed_data = zlib.compress(raw_data, level=9)
    idat = make_chunk(b"IDAT", compressed_data)
    iend = make_chunk(b"IEND", b"")
    png_data = png_signature + ihdr + idat + iend

    with open(path, "wb") as image_file:
        image_file.write(png_data)
    
    return path

def print_result(step, expected_status, response, is_secure):
    icon = "✅ [PASS]" if is_secure else "❌ [FAIL]"
    print(f"{icon} {step} | Expected: {expected_status} -> Actual: {response.status_code}")
    print(f"  URL: {response.request.method} {response.url}")
    print("  Response Body:")
    try:
        print(json.dumps(response.json(), indent=2, ensure_ascii=False))
    except ValueError:
        text = response.text.strip()
        print(f"    {text}" if text else "    <empty>")
    print()

def print_skip(step, reason):
    print(f"⏭️ [SKIP] {step}")
    print(f"  Reason: {reason}")
    print()
