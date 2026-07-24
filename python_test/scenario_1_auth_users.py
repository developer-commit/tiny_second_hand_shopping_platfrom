import requests
import uuid
import sys
import os
from utils import print_result, print_skip

BASE_URL = "http://localhost:8080/v1"

def run_scenario():
    print("=== Scenario 1: Auth & Users ===")
    session = requests.Session()
    account_id = f"test_user_{uuid.uuid4().hex[:8]}"
    secret_key = "secure_password_123"

    # Step 1.0: Send Code
    print("\n--- [Step 1.0] Send Code ---")
    response = session.post(
        f"{BASE_URL}/auth/sendcode",
        json={
            "account_id": account_id,
            "contact_email": f"{account_id}@example.com",
            "verification_code": "000000"
        }
    )
    print_result("Send Code", "201", response, response.status_code == 201)

    # Step 1.1: Signup
    print("\n--- [Step 1.1] Signup ---")
    response = session.post(
        f"{BASE_URL}/auth/signup",
        json={
            "account_id": account_id,
            "secret_key": secret_key,
            "contact_email": f"{account_id}@example.com",
            "verification_code": "000000"
        }
    )
    print_result("Signup", "201", response, response.status_code in [201, 400, 409])

    # Step 1.2: Login
    print("\n--- [Step 1.2] Login ---")
    response = session.post(
        f"{BASE_URL}/auth/login",
        json={"account_id": account_id, "secret_key": secret_key}
    )
    if response.status_code == 200:
        json_resp = response.json()
        access_token = json_resp.get("access_token") or json_resp.get("data", {}).get("access_token")
        if access_token:
            session.headers.update({"Authorization": f"Bearer {access_token}"})
            print_result("Login & Get Token", "200", response, True)
            
            # Save token for other scenarios
            with open("test_token.txt", "w") as f:
                f.write(access_token)
        else:
            print_result("Login & Get Token", "200", response, False)
            return False
    else:
        print_result("Login", "200", response, False)
        return False

    # Step 1.3: Get My Profile
    print("\n--- [Step 1.3] Get My Profile ---")
    response = session.get(f"{BASE_URL}/users/me")
    print_result("Get My Profile", "200", response, response.status_code == 200)
    if response.status_code == 200:
        json_resp = response.json()
        user_uid = json_resp.get("user_uid") or json_resp.get("data", {}).get("user_uid")
        with open("test_user.txt", "w") as f:
            f.write(user_uid)

    # Step 1.4: Update Profile
    print("\n--- [Step 1.4] Update Profile ---")
    response = session.patch(
        f"{BASE_URL}/users/me",
        json={"display_name": f"NewName_{account_id}", "bio": "Hello World"}
    )
    print_result("Update Profile", "200", response, response.status_code == 200)

    # Step 1.5: 2FA Setup
    print("\n--- [Step 1.5] 2FA Setup ---")
    response = session.post(f"{BASE_URL}/users/me/2fa/setup")
    print_result("2FA Setup", "200", response, response.status_code == 200)

    return True

if __name__ == "__main__":
    success = run_scenario()
    sys.exit(0 if success else 1)
