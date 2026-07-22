import requests
import uuid
import json
import struct
import zlib
import os


BASE_URL = "http://localhost:8080/v1"


# =========================================================
# 테스트용 PNG 이미지 생성
# =========================================================
def create_test_png(
    path="test_image.png",
    width=256,
    height=256,
):
    raw_rows = []

    for _ in range(height):
        row = b"\x00"

        # RGBA
        pixel = bytes([
            220,
            220,
            220,
            255,
        ])

        row += pixel * width
        raw_rows.append(row)

    raw_data = b"".join(raw_rows)

    def make_chunk(
        chunk_type,
        data,
    ):
        length = struct.pack(
            ">I",
            len(data),
        )

        crc = zlib.crc32(
            chunk_type + data
        ) & 0xFFFFFFFF

        return (
            length
            + chunk_type
            + data
            + struct.pack(
                ">I",
                crc,
            )
        )

    png_signature = (
        b"\x89PNG\r\n\x1a\n"
    )

    ihdr_data = struct.pack(
        ">IIBBBBB",
        width,
        height,
        8,
        6,
        0,
        0,
        0,
    )

    ihdr = make_chunk(
        b"IHDR",
        ihdr_data,
    )

    compressed_data = zlib.compress(
        raw_data,
        level=9,
    )

    idat = make_chunk(
        b"IDAT",
        compressed_data,
    )

    iend = make_chunk(
        b"IEND",
        b"",
    )

    png_data = (
        png_signature
        + ihdr
        + idat
        + iend
    )

    with open(
        path,
        "wb",
    ) as image_file:

        image_file.write(
            png_data
        )

    file_size = len(png_data)

    print(
        f"  테스트 이미지 생성: "
        f"{path} "
        f"({file_size / 1024:.2f} KB)"
    )

    if file_size >= 5 * 1024 * 1024:
        raise ValueError(
            "테스트 이미지가 5MB를 초과했습니다."
        )

    return path


class UserScenarioTester:

    def __init__(
        self,
        account_id,
        secret_key,
    ):

        self.session = requests.Session()

        self.account_id = account_id
        self.secret_key = secret_key

        self.access_token = None

        self.item_uid = None
        self.trade_uid = None
        self.room_uid = None
        self.noti_uid = None
        self.user_uid = None

        self.results = []

    # =====================================================
    # 공통 출력
    # =====================================================
    def print_result(
        self,
        step,
        expected_status,
        response,
        is_secure,
    ):

        icon = (
            "✅ [정상]"
            if is_secure
            else "❌ [취약점 발견/에러]"
        )

        print(
            f"{icon} {step} "
            f"| 예상: {expected_status} "
            f"-> 실제: {response.status_code}"
        )

        print(
            f"  URL: "
            f"{response.request.method} "
            f"{response.url}"
        )

        print(
            "  Response Headers:"
        )

        for key, value in response.headers.items():

            print(
                f"    {key}: {value}"
            )

        print(
            "  Response Body:"
        )

        try:

            body = response.json()

            print(
                json.dumps(
                    body,
                    indent=2,
                    ensure_ascii=False,
                )
            )

        except ValueError:

            text = (
                response.text.strip()
            )

            if text:

                print(
                    f"    {text}"
                )

            else:

                print(
                    "    <empty>"
                )

        print()

        self.results.append(
            {
                "step": step,
                "expected": expected_status,
                "actual": response.status_code,
                "passed": is_secure,
            }
        )

    # =====================================================
    # SKIP 출력
    # =====================================================
    def print_skip(
        self,
        step,
        reason,
    ):

        print(
            f"⏭️ [SKIP] {step}"
        )

        print(
            f"  사유: {reason}"
        )

        print()

    # =====================================================
    # 1. 회원가입
    # =====================================================
    def step1_signup(self):

        print(
            "\n--- [Step 1] 회원가입 ---"
        )

        response = self.session.post(
            f"{BASE_URL}/auth/signup",
            json={
                "account_id":
                    self.account_id,

                "secret_key":
                    self.secret_key,

                "contact_email":
                    f"{self.account_id}@test.com",

                "contact_phone":
                    "010-1234-5678",

                "verification_code":
                    "123456",
            },
        )

        self.print_result(
            "회원가입",
            "201",
            response,
            response.status_code
            in [
                201,
                400,
                409,
            ],
        )

    # =====================================================
    # 2. 로그인
    # =====================================================
    def step2_login(self):

        print(
            "\n--- [Step 2] 로그인 ---"
        )

        response = self.session.post(
            f"{BASE_URL}/auth/login",
            json={
                "account_id":
                    self.account_id,

                "secret_key":
                    self.secret_key,
            },
        )

        if response.status_code != 200:

            self.print_result(
                "로그인",
                "200",
                response,
                False,
            )

            return False

        try:

            data = response.json()

            self.access_token = (
                data.get(
                    "access_token"
                )
            )

        except ValueError:

            self.access_token = None

        if not self.access_token:

            self.print_result(
                "로그인 및 access_token 획득",
                "200 + access_token",
                response,
                False,
            )

            return False

        self.session.headers.update(
            {
                "Authorization":
                    f"Bearer "
                    f"{self.access_token}"
            }
        )

        self.print_result(
            "로그인 및 access_token 획득",
            "200",
            response,
            True,
        )

        return True

    # =====================================================
    # 3. 사용자 API
    # =====================================================
    def step3_user_apis(self):

        print(
            "\n--- [Step 3] 사용자 API 테스트 ---"
        )

        # ---------------------------------------------
        # GET /users/me
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/users/me"
        )

        if response.status_code == 200:
            try:
                self.user_uid = response.json().get("user_uid")
            except ValueError:
                pass

        self.print_result(
            "내 프로필 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # PATCH /users/me
        # ---------------------------------------------
        response = self.session.patch(
            f"{BASE_URL}/users/me",
            json={
                "display_name":
                    f"TestUser_{self.account_id}",

                "bio":
                    "API Security Test User",
            },
        )

        self.print_result(
            "내 프로필 수정",
            "200",
            response,
            response.status_code == 200,
        )

    # =====================================================
    # 4. 2FA API
    # =====================================================
    def step4_2fa_apis(self):

        print(
            "\n--- [Step 4] 2FA API 테스트 ---"
        )

        # ---------------------------------------------
        # POST /users/me/2fa/setup
        # ---------------------------------------------
        response = self.session.post(
            f"{BASE_URL}/users/me/2fa/setup"
        )

        self.print_result(
            "2FA 설정 요청",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # OTP가 실제로 필요하므로 enable은
        # setup 결과에 OTP가 없는 경우 SKIP
        # ---------------------------------------------
        self.print_skip(
            "2FA 활성화",
            "실제 OTP 생성기 또는 사용자가 입력한 "
            "otp_token이 필요합니다.",
        )

    # =====================================================
    # 5. 상품 이미지 업로드
    # =====================================================
    def step5_upload_image(self):

        print(
            "\n--- [Step 5] 이미지 업로드 ---"
        )

        image_path = (
            "test_image.png"
        )

        try:

            create_test_png(
                image_path
            )

            with open(
                image_path,
                "rb",
            ) as image_file:

                response = (
                    self.session.post(
                        f"{BASE_URL}/uploads",

                        files={
                            "file": (
                                "test_image.png",
                                image_file,
                                "image/png",
                            )
                        },
                    )
                )

        except Exception as e:

            print(
                f"❌ 이미지 업로드 예외: {e}"
            )

            return None

        self.print_result(
            "이미지 업로드",
            "201",
            response,
            response.status_code == 201,
        )

        if response.status_code != 201:

            return None

        try:

            return response.json().get(
                "image_url"
            )

        except ValueError:

            return None

    # =====================================================
    # 6. 상품 API
    # =====================================================
    def step6_product_apis(
        self,
        image_url,
    ):

        print(
            "\n--- [Step 6] 상품 API 테스트 ---"
        )

        # ---------------------------------------------
        # POST /products
        # ---------------------------------------------
        response = self.session.post(
            f"{BASE_URL}/products",
            json={
                "heading":
                    "API Security Test Item",

                "detail_body":
                    "Security test product",

                "asking_price":
                    0.5,

                "group_category":
                    "electronics",

                "item_tags":
                    [
                        "test",
                        "security",
                    ],

                "image_urls":
                    [
                        image_url
                    ]
                    if image_url
                    else [],
            },
        )

        self.print_result(
            "상품 등록",
            "201",
            response,
            response.status_code == 201,
        )

        if response.status_code == 201:

            try:

                data = response.json()

                self.item_uid = (
                    data.get(
                        "item_uid"
                    )
                )

            except ValueError:

                pass

        if not self.item_uid:

            self.print_skip(
                "상품 단건 조회",
                "상품 등록 응답에서 item_uid를 "
                "확인하지 못했습니다.",
            )

            return

        print(
            f"  생성된 item_uid: "
            f"{self.item_uid}"
        )

        # ---------------------------------------------
        # GET /products/{item_uid}
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/products/"
            f"{self.item_uid}"
        )

        self.print_result(
            "상품 단건 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # PATCH /products/{item_uid}/status
        # ---------------------------------------------
        response = self.session.patch(
            f"{BASE_URL}/products/"
            f"{self.item_uid}/status",

            json={
                "current_state":
                    "reserved",
            },
        )

        self.print_result(
            "상품 상태 변경",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # GET /products
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/products"
        )

        self.print_result(
            "상품 목록 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # GET /products?keyword=
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/products",
            params={
                "keyword":
                    "API Security Test"
            },
        )

        self.print_result(
            "상품 키워드 검색",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # GET /products?group_category=
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/products",
            params={
                "group_category":
                    "electronics"
            },
        )

        self.print_result(
            "상품 카테고리 검색",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # GET /products?item_tag=
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/products",
            params={
                "item_tag":
                    "security"
            },
        )

        self.print_result(
            "상품 태그 검색",
            "200",
            response,
            response.status_code == 200,
        )

    # =====================================================
    # 7. Wallet API
    # =====================================================
    def step7_wallet_apis(self):

        print(
            "\n--- [Step 7] Wallet API 테스트 ---"
        )

        # ---------------------------------------------
        # GET /wallet
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/wallet"
        )

        self.print_result(
            "내 지갑 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # GET /wallet/history
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/wallet/history"
        )

        self.print_result(
            "지갑 거래내역 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # 실제 출금은 테스트하지 않음
        # 이유:
        # - 실제 BCH 주소 필요
        # - 실제 잔액 필요
        # - 실제 OTP 필요
        # ---------------------------------------------
        self.print_skip(
            "외부 지갑 출금",
            "실제 BCH 주소/잔액/OTP가 필요한 "
            "실제 자산 이동 API이므로 자동 호출하지 않습니다.",
        )

    # =====================================================
    # 8. Chat API
    # =====================================================
    def step8_chat_apis(self):

        print(
            "\n--- [Step 8] Chat API 테스트 ---"
        )

        if not self.item_uid:

            self.print_skip(
                "채팅방 생성",
                "상품 UID가 없습니다.",
            )

            return

        # ---------------------------------------------
        # POST /chat/rooms
        # ---------------------------------------------
        response = self.session.post(
            f"{BASE_URL}/chat/rooms",
            json={
                "item_uid":
                    self.item_uid,
                "partner_uid":
                    self.user_uid
            },
        )

        self.print_result(
            "상품 채팅방 생성/조회",
            "200",
            response,
            response.status_code == 200,
        )

        if response.status_code == 200:

            try:

                self.room_uid = (
                    response.json().get(
                        "room_uid"
                    )
                )

            except ValueError:

                pass

        if not self.room_uid:

            self.print_skip(
                "채팅방 내역 조회",
                "응답에서 room_uid를 확인하지 못했습니다.",
            )

            return

        # ---------------------------------------------
        # GET /chat/rooms/{room_uid}/history
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/chat/rooms/"
            f"{self.room_uid}/history"
        )

        self.print_result(
            "채팅 내역 조회",
            "200",
            response,
            response.status_code == 200,
        )

    # =====================================================
    # 9. Escrow API
    # =====================================================
    def step9_escrow_apis(self):

        print(
            "\n--- [Step 9] Escrow API 테스트 ---"
        )

        if not self.item_uid:

            self.print_skip(
                "에스크로 예치",
                "상품 UID가 없습니다.",
            )

            return

        # ---------------------------------------------
        # PATCH /products/{item_uid}/status to on_sale
        # ---------------------------------------------
        self.session.patch(
            f"{BASE_URL}/products/"
            f"{self.item_uid}/status",
            json={"current_state": "on_sale"},
        )

        # ---------------------------------------------
        # POST /escrow
        # ---------------------------------------------
        response = self.session.post(
            f"{BASE_URL}/escrow",
            json={
                "item_uid":
                    self.item_uid
            },
        )

        self.print_result(
            "에스크로 예치",
            "201",
            response,
            response.status_code == 201,
        )

        if response.status_code == 201:

            try:

                self.trade_uid = (
                    response.json().get(
                        "trade_uid"
                    )
                )

            except ValueError:

                pass

        # 거래 UID가 없으면 이후 테스트 불가
        if not self.trade_uid:

            self.print_skip(
                "에스크로 확정/분쟁",
                "에스크로 응답에서 trade_uid를 "
                "확인하지 못했습니다.",
            )

            return

        # ---------------------------------------------
        # 실제 거래를 생성했으므로
        # 테스트 환경에 따라 confirm/dispute 중
        # 하나를 호출해야 합니다.
        #
        # 여기서는 실제 자금 정산 가능성이 있으므로
        # 자동 호출하지 않습니다.
        # ---------------------------------------------
        self.print_skip(
            "에스크로 확정",
            "실제 거래 정산이 발생할 수 있어 "
            "자동 호출하지 않습니다.",
        )

        self.print_skip(
            "에스크로 분쟁",
            "실제 거래 상태를 변경하므로 "
            "자동 호출하지 않습니다.",
        )

    # =====================================================
    # 10. Review API
    # =====================================================
    def step10_review_api(self):

        print(
            "\n--- [Step 10] Review API 테스트 ---"
        )

        if not self.trade_uid:

            self.print_skip(
                "리뷰 작성",
                "trade_uid가 없습니다.",
            )

            return

        # 리뷰는 거래 완료 상태가 필요하므로
        # 임의로 호출하면 정상적으로 4xx가 나올 수 있음
        self.print_skip(
            "리뷰 작성",
            "거래 완료 상태가 필요하므로 "
            "실제 완료된 trade_uid가 필요합니다.",
        )

    # =====================================================
    # 11. 신고 API
    # =====================================================
    def step11_report_api(self):

        print(
            "\n--- [Step 11] 신고 API 테스트 ---"
        )

        if not self.item_uid:

            self.print_skip(
                "상품 신고",
                "상품 UID가 없습니다.",
            )

            return

        response = self.session.post(
            f"{BASE_URL}/products/"
            f"{self.item_uid}/reports",

            json={
                "cause":
                    "API Security Test Report",
            },
        )

        self.print_result(
            "상품 신고",
            "400",
            response,
            response.status_code == 400,
        )

    # =====================================================
    # 12. Notification API
    # =====================================================
    def step12_notification_apis(self):

        print(
            "\n--- [Step 12] Notification API 테스트 ---"
        )

        # ---------------------------------------------
        # GET /notifications
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/notifications"
        )

        self.print_result(
            "내 알림 조회",
            "200",
            response,
            response.status_code == 200,
        )

        # ---------------------------------------------
        # 알림 UID 확인
        # ---------------------------------------------
        if response.status_code == 200:

            try:

                notifications = (
                    response.json()
                )

                if (
                    isinstance(
                        notifications,
                        list,
                    )
                    and notifications
                ):

                    self.noti_uid = (
                        notifications[0].get(
                            "noti_uid"
                        )
                    )

            except ValueError:

                pass

        if not self.noti_uid:

            self.print_skip(
                "알림 읽음 처리",
                "읽지 않은 알림 또는 noti_uid가 없습니다.",
            )

            return

        # ---------------------------------------------
        # PATCH /notifications/{noti_uid}/read
        # ---------------------------------------------
        response = self.session.patch(
            f"{BASE_URL}/notifications/"
            f"{self.noti_uid}/read"
        )

        self.print_result(
            "알림 읽음 처리",
            "200",
            response,
            response.status_code == 200,
        )

    # =====================================================
    # 13. 보안 테스트
    # =====================================================
    def step13_security_checks(self):

        print(
            "\n--- [Step 13] 보안 테스트 ---"
        )

        # ---------------------------------------------
        # 관리자 권한 상승
        # ---------------------------------------------
        response = self.session.get(
            f"{BASE_URL}/admin/stats"
        )

        self.print_result(
            "일반 유저 관리자 통계 접근",
            "403",
            response,
            response.status_code == 403,
        )

        # ---------------------------------------------
        # 관리자 강제 정산
        # ---------------------------------------------
        response = self.session.post(
            f"{BASE_URL}/admin/"
            f"escrow/force-settle",

            json={
                "trade_uid":
                    "fake_trade_uid",

                "settle_to":
                    "buyer",

                "reason":
                    "Unauthorized security test",
            },
        )

        self.print_result(
            "일반 유저 관리자 강제 정산 접근",
            "403",
            response,
            response.status_code == 403,
        )

        # ---------------------------------------------
        # IDOR
        # ---------------------------------------------
        fake_item_uid = (
            "some_other_users_item_uid_12345"
        )

        response = self.session.patch(
            f"{BASE_URL}/products/"
            f"{fake_item_uid}/status",

            json={
                "current_state":
                    "sold"
            },
        )

        self.print_result(
            "타인 상품 상태 변경 IDOR",
            "403 or 404",
            response,
            response.status_code
            in [
                403,
                404,
            ],
        )

    # =====================================================
    # 14. 인증 없는 접근
    # =====================================================
    def step14_unauthorized_checks(self):

        print(
            "\n--- [Step 14] 인증 없는 접근 테스트 ---"
        )

        unauth_session = (
            requests.Session()
        )

        protected_endpoints = [

            (
                "내 프로필",
                "GET",
                f"{BASE_URL}/users/me",
            ),

            (
                "내 지갑",
                "GET",
                f"{BASE_URL}/wallet",
            ),

            (
                "지갑 거래내역",
                "GET",
                f"{BASE_URL}/wallet/history",
            ),

            (
                "알림",
                "GET",
                f"{BASE_URL}/notifications",
            ),

        ]

        for (
            name,
            method,
            url,
        ) in protected_endpoints:

            response = (
                unauth_session.request(
                    method,
                    url,
                )
            )

            self.print_result(
                f"인증 없는 {name} 접근",
                "401",
                response,
                response.status_code == 401,
            )

    # =====================================================
    # 15. 테스트 결과 요약
    # =====================================================
    def print_summary(self):

        print(
            "\n"
            + "=" * 60
        )

        print(
            "📊 API 테스트 결과 요약"
        )

        print(
            "=" * 60
        )

        total = len(
            self.results
        )

        passed = sum(
            1
            for result
            in self.results
            if result["passed"]
        )

        failed = (
            total
            - passed
        )

        print(
            f"총 테스트: {total}"
        )

        print(
            f"정상: {passed}"
        )

        print(
            f"실패/취약점: {failed}"
        )

        if failed == 0:

            print(
                "\n✅ 모든 실행된 API 테스트가 "
                "예상 결과를 만족했습니다."
            )

        else:

            print(
                "\n⚠️ 실패 또는 취약점이 발견되었습니다."
            )

        print(
            "=" * 60
        )


# =========================================================
# 실행부
# =========================================================
if __name__ == "__main__":

    # 매 실행마다 새로운 계정
    random_id = (
        "testuser_"
        f"{uuid.uuid4().hex[:12]}"
    )

    print(
        "=" * 60
    )

    print(
        "🕵️‍♂️ API 보안 및 기능 통합 테스트 시작"
    )

    print(
        "=" * 60
    )

    print(
        f"테스트 계정: {random_id}"
    )

    tester = UserScenarioTester(
        account_id=random_id,
        secret_key="P@ssw0rd123!",
    )

    # -------------------------------------------------
    # 1. 회원가입
    # -------------------------------------------------
    tester.step1_signup()

    # -------------------------------------------------
    # 2. 로그인 성공 시 인증 API 테스트
    # -------------------------------------------------
    if tester.step2_login():

        # 사용자
        tester.step3_user_apis()

        # 2FA
        tester.step4_2fa_apis()

        # 이미지 업로드
        image_url = (
            tester.step5_upload_image()
        )

        # 상품
        tester.step6_product_apis(
            image_url
        )

        # 지갑
        tester.step7_wallet_apis()

        # 채팅
        tester.step8_chat_apis()

        # 에스크로
        tester.step9_escrow_apis()

        # 리뷰
        tester.step10_review_api()

        # 신고
        tester.step11_report_api()

        # 알림
        tester.step12_notification_apis()

        # 권한 / IDOR
        tester.step13_security_checks()

    else:

        print(
            "\n❌ 로그인 실패"
        )

        print(
            "인증이 필요한 API 테스트를 "
            "진행할 수 없습니다."
        )

    # -------------------------------------------------
    # 인증 없는 접근은 로그인 여부와 무관하게 테스트
    # -------------------------------------------------
    tester.step14_unauthorized_checks()

    # -------------------------------------------------
    # 결과
    # -------------------------------------------------
    tester.print_summary()

    print(
        "\n🏁 API 통합 테스트 종료"
    )