from typing import Optional, Dict, Any
import requests

# ==============================================================================
# CONFIG & CONSTANTS
# ==============================================================================
BASE_URL = "http://localhost:8080"

TEST_USERS = [
    {
        "account_id": "admin",
        "contact_email": "admin@gmail.com",
        "contact_phone": None,
        "secret_key": "22222222",
        "verification_code": "000000"
    },
    {
        "account_id": "test1",
        "contact_email": "shiroi.py1@gmail.com",
        "contact_phone": None,
        "secret_key": "11111111",
        "verification_code": "000000"
    },
    {
        "account_id": "test2",
        "contact_email": "shiroi.py2@gmail.com",
        "contact_phone": None,
        "secret_key": "22222222",
        "verification_code": "000000"
    }
]

SAMPLE_PRODUCT = {
    "asking_price": 1,
    "currency": "E_T_H",
    "detail_body": "dsfaffadf",
    "group_category": "electronics",
    "heading": "test",
    "image_urls": [],
    "item_tags": []
}

def fund_wallet_anvil(to_address: str, amount_eth: float = 100.0) -> bool:
    """Anvil 마스터 계정에서 대상 지갑 주소로 ETH를 송금합니다."""
    ANVIL_RPC_URL = "http://localhost:8545"  # 사용 중인 Anvil RPC Port로 필요시 수정
    MASTER_ACCOUNT = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"  # Anvil 기본 0번 마스터 계정

    wei_value = hex(int(amount_eth * 10**18))
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_sendTransaction",
        "params": [{
            "from": MASTER_ACCOUNT,
            "to": to_address,
            "value": wei_value
        }],
        "id": 1
    }

    try:
        res = requests.post(ANVIL_RPC_URL, json=payload)
        res_json = res.json()
        if "result" in res_json:
            print(f"  └─ [SUCCESS] Funded! Tx: {res_json['result']}")
            return True
        print(f"  └─ [Failed] Funding failed: {res_json}")
        return False
    except Exception as e:
        print(f"  └─ [Error] Anvil RPC 연결 실패: {e}")
        return False

# ==============================================================================
# API CLIENT SERVICES
# ==============================================================================
def request_sendcode(session: requests.Session, user: Dict[str, Any]) -> bool:
    """인증 코드 발송 요청 (POST /v1/auth/sendcode)"""
    payload = {
        "account_id": user["account_id"],
        "contact_email": user["contact_email"],
        "contact_phone": user["contact_phone"]
    }
    res = session.post(f"{BASE_URL}/v1/auth/sendcode", json=payload)
    
    if res.status_code in [200, 201]:
        print("  └─ [Success] sendcode")
        return True
    print(f"  └─ [Failed] sendcode ({res.status_code}): {res.text}")
    return False


def request_signup(session: requests.Session, user: Dict[str, Any]) -> bool:
    """회원가입 요청 (POST /v1/auth/signup)"""
    res = session.post(f"{BASE_URL}/v1/auth/signup", json=user)
    
    if res.status_code in [200, 201]:
        print("  └─ [Success] signup")
        return True
    print(f"  └─ [Failed] signup ({res.status_code}): {res.text}")
    return False


def request_login(session: requests.Session, user: Dict[str, Any]) -> bool:
    """로그인 요청 및 JWT Authorization 헤더 주입 (POST /v1/auth/login)"""
    payload = {
        "account_id": user["account_id"],
        "secret_key": user["secret_key"]
    }
    res = session.post(f"{BASE_URL}/v1/auth/login", json=payload)
    
    if res.status_code not in [200, 201]:
        print(f"  └─ [Failed] login ({res.status_code}): {res.text}")
        return False

    try:
        data = res.json().get("data", {})
        token = data.get("access_token")
        token_type = data.get("token_type", "Bearer")
        
        if token:
            session.headers.update({"Authorization": f"{token_type} {token}"})
            print("  └─ [Success] login (Authorization 헤더 주입 완료)")
            return True
        
        print("  └─ [Error] 응답 데이터에 access_token이 없습니다.")
        return False
    except Exception as e:
        print(f"  └─ [Error] 로그인 토큰 파싱 실패: {e}")
        return False


def fetch_wallet_info(session: requests.Session, account_id: str) -> Optional[Dict[str, Any]]:
    """지갑 정보 조회 (GET /v1/wallet)"""
    res = session.get(f"{BASE_URL}/v1/wallet")
    
    if res.status_code == 200:
        return res.json()
    
    print(f"  └─ [Failed] wallet 조회 ({res.status_code}): {res.text}")
    return None


def create_product(session: requests.Session, product_payload: Dict[str, Any]) -> bool:
    """상품 등록 요청 (POST /v1/products)"""
    res = session.post(f"{BASE_URL}/v1/products", json=product_payload)
    
    if res.status_code in [200, 201]:
        print("  └─ [Success] 상품 등록 완료!")
        return True
    
    print(f"  └─ [Failed] 상품 등록 실패 ({res.status_code}): {res.text}")
    return False


# ==============================================================================
# DISPLAY HELPERS
# ==============================================================================
def print_wallet_box(account_id: str, wallet: Dict[str, Any]) -> None:
    """지갑 정보를 터미널에 정돈된 형태로 출력합니다."""
    print("\n┌" + "─" * 48 + "┐")
    print(f"│ 계정 ID         : {account_id:<31} │")
    print(f"│ Public Address  : {str(wallet.get('public_address', 'N/A')):<31} │")
    print(f"│ Available       : {str(wallet.get('available_balance', 0.0)):<31} │")
    print(f"│ Locked Escrow   : {str(wallet.get('locked_in_escrow', 0.0)):<31} │")
    print(f"│ ETH Balance     : {str(wallet.get('eth_balance', 0.0)):<31} │")
    print("└" + "─" * 48 + "┘")


# ==============================================================================
# PIPELINE EXECUTION
# ==============================================================================
def process_user_account(user: Dict[str, Any], is_first_account: bool) -> None:
    """단일 계정에 대한 회원가입, 로그인, 지갑 조회 및 조건부 상품 등록 처리"""
    session = requests.Session()
    account_id = user["account_id"]
    
    print(f"\n==========================================")
    print(f"👤 [{account_id}] 계정 자동화 작업 시작")
    print(f"==========================================")

    # 1. 인증 코드 발송
    if not request_sendcode(session, user):
        return

    # 2. 회원가입
    if not request_signup(session, user):
        return

    # 3. 로그인
    if not request_login(session, user):
        return

    # 4. 지갑 정보 조회 및 출력
    wallet_data = fetch_wallet_info(session, account_id)
    if wallet_data:
        print_wallet_box(account_id, wallet_data)
        
        # [추가] test2 계정인 경우 지갑 주소를 추출하여 100 ETH 송금
        if account_id == "test2":
            # API 응답 구조에 맞춰 public_address 추출 (root 또는 data 내부)
            target_address = wallet_data.get("public_address") or wallet_data.get("data", {}).get("public_address")
            if target_address:
                print(f"\n💸 [{account_id}] 100 ETH 송금 진행 중...")
                fund_wallet_anvil(target_address, 100)

    # 5. 계정 1인 경우에만 상품 등록 수행
    if is_first_account:
        print(f"\n📦 [{account_id}] 계정 상품 등록 진행...")
        create_product(session, SAMPLE_PRODUCT)


def main():
    """메인 실행 함수"""
    for index, user in enumerate(TEST_USERS):
        is_first_account = (index == 0)
        process_user_account(user, is_first_account)


if __name__ == "__main__":
    main()