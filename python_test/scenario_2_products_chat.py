import requests
import sys
import os
from utils import create_test_png, print_result, print_skip

BASE_URL = "http://localhost:8080/v1"

def run_scenario():
    print("=== Scenario 2: Products & Chat ===")
    
    # Load token
    try:
        with open("test_token.txt", "r") as f:
            access_token = f.read().strip()
    except FileNotFoundError:
        print("❌ Cannot find test_token.txt. Run Scenario 1 first.")
        return False

    session = requests.Session()
    session.headers.update({"Authorization": f"Bearer {access_token}"})

    # Step 2.1: Upload Image
    print("\n--- [Step 2.1] Upload Image ---")
    image_path = create_test_png()
    image_url = None
    with open(image_path, "rb") as image_file:
        response = session.post(
            f"{BASE_URL}/uploads",
            files={"file": ("test_image.png", image_file, "image/png")}
        )
        print_result("Image Upload", "201", response, response.status_code == 201)
        if response.status_code == 201:
            json_resp = response.json()
            image_url = json_resp.get("image_url") or json_resp.get("data", {}).get("image_url")

    # Step 2.2: Create Product
    print("\n--- [Step 2.2] Create Product ---")
    response = session.post(
        f"{BASE_URL}/products",
        json={
            "heading": "Scenario 2 Test Item",
            "detail_body": "This is a test product for scenario 2",
            "asking_price": 100.5,
            "currency": "B_C_H",
            "group_category": "electronics",
            "item_tags": ["test", "scenario2"],
            "image_urls": [image_url] if image_url else []
        }
    )
    print_result("Create Product", "201", response, response.status_code == 201)
    
    item_uid = None
    if response.status_code == 201:
        json_resp = response.json()
        item_uid = json_resp.get("item_uid") or json_resp.get("data", {}).get("item_uid")
        with open("test_item.txt", "w") as f:
            f.write(item_uid)

    if not item_uid:
        print_skip("Product Details", "No item_uid obtained.")
        return False

    # Step 2.3: Get Product Details
    print("\n--- [Step 2.3] Get Product Details ---")
    response = session.get(f"{BASE_URL}/products/{item_uid}")
    print_result("Get Product Details", "200", response, response.status_code == 200)

    # Step 2.4: Change Product Status
    print("\n--- [Step 2.4] Change Product Status ---")
    response = session.patch(
        f"{BASE_URL}/products/{item_uid}/status",
        json={"current_state": "reserved"}
    )
    print_result("Change Product Status", "200", response, response.status_code == 200)

    # Step 2.5: Search Products
    print("\n--- [Step 2.5] Search Products ---")
    response = session.get(f"{BASE_URL}/products", params={"keyword": "Scenario 2"})
    print_result("Search Products by Keyword", "200", response, response.status_code == 200)

    # Load user uid
    try:
        with open("test_user.txt", "r") as f:
            user_uid = f.read().strip()
    except FileNotFoundError:
        user_uid = "dummy_uid"

    # Step 2.6: Create Chat Room
    print("\n--- [Step 2.6] Create Chat Room ---")
    response = session.post(
        f"{BASE_URL}/chat/rooms",
        json={"item_uid": item_uid, "partner_uid": user_uid}
    )
    print_result("Create Chat Room", "200", response, response.status_code == 200)
    
    if response.status_code == 200:
        json_resp = response.json()
        room_uid = json_resp.get("room_uid") or json_resp.get("data", {}).get("room_uid")
        if room_uid:
            print("\n--- [Step 2.7] Chat History ---")
            history_resp = session.get(f"{BASE_URL}/chat/rooms/{room_uid}/history")
            print_result("Chat History", "200", history_resp, history_resp.status_code == 200)

    return True

if __name__ == "__main__":
    success = run_scenario()
    sys.exit(0 if success else 1)
