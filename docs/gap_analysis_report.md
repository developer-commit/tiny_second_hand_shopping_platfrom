# API & DTO Gap Analysis Report

## 1. Executive Summary
This report provides a cross-comparison between the standard `openapi.yaml` specification, the implemented backend endpoints & shared DTOs (`crates/shared/src/dto/`), and the frontend mock API calls. 
Overall, the architectural boundaries and obfuscation principles are correctly implemented across all three layers. However, several discrepancies exist between the `openapi.yaml` and the Rust `shared/dto` definitions, mostly involving missing fields in the OpenAPI spec and naming inconsistencies.

- **Total Endpoints Checked:** 21
- **Overall Alignment:** Mostly aligned in routing and structure, but significant DTO mismatches exist.
- **Critical Blocking Issues:** The `GET /products` endpoint return type and `GET /wallet/history` return type diverge drastically in schema naming.

## 2. Endpoint & Path Mapping Table

| Functionality | OpenAPI Path & Method | Backend Endpoint | Frontend Call / Mock | Status (Match / Mismatch / Missing) |
|---|---|---|---|---|
| **Signup** | `POST /auth/signup` | `auth_handler::signup` | `signup.rs` | Match |
| **Login** | `POST /auth/login` | `auth_handler::login` | `login.rs` | Match |
| **My Profile** | `GET /users/me` | `user_handler::get_my_profile` | `mypage.rs` | Match |
| **2FA Setup** | `POST /users/me/2fa/setup` | `auth_handler::setup_2fa` | `Not Yet Integrated` | Match (Backend) |
| **Product List** | `GET /products` | `product_handler::list_products` | `home.rs` | **Mismatch** (Return Type) |
| **Create Product** | `POST /products` | `product_handler::create_product` | `product_new.rs` | **Mismatch** (Payload) |
| **Product Detail** | `GET /products/{item_uid}` | `product_handler::get_product` | `product_detail.rs` | Match |
| **Product Status** | `PATCH /products/{item_uid}/status` | `product_handler::update_product_status`| `product_edit.rs` | Match |
| **Wallet Info** | `GET /wallet` | `wallet_handler::get_wallet` | `wallet.rs` | Match |
| **Wallet Withdraw**| `POST /wallet/withdraw` | `wallet_handler::withdraw` | `wallet.rs` | Match |
| **Wallet History** | `GET /wallet/history` | `wallet_handler::get_tx_history`| `wallet.rs` | **Mismatch** (Payload) |
| **Escrow Init** | `POST /escrow` | `escrow_handler::initiate_escrow` | `escrow.rs` | Match |
| **Escrow Status** | `GET /escrow/{trade_uid}` | *Missing in OpenAPI* | *Not found* | **Missing** |

## 3. DTO & Field Discrepancies

### 3.1 Product DTO Mismatches
**Location**: `product_dto.rs` vs `openapi.yaml`
- **OpenAPI Spec**: `GET /products` returns `array` of `ItemDetailRes`.
- **Backend/Frontend**: Uses `ItemSummaryRes` for list views.
- **Impact & Recommendation**: The frontend uses `thumbnail_url` in the list view which doesn't exist in `ItemDetailRes`. **Update OpenAPI** to define `ItemSummaryRes` and use it for `GET /products`.

**Location**: `CreateItemReq` (POST `/products`)
- **OpenAPI Spec**: Missing `image_urls`.
- **Backend/Frontend**: Includes `pub image_urls: Vec<String>`.
- **Impact & Recommendation**: Frontend cannot legally send `image_urls` per OpenAPI strict validation. **Update OpenAPI** to include `image_urls`.

**Location**: `ItemDetailRes` (GET `/products/{item_uid}`)
- **OpenAPI Spec**: Missing `listed_at` (String).
- **Backend/Frontend**: Includes `pub listed_at: String`.
- **Impact & Recommendation**: Minor. Update OpenAPI to reflect reality.

### 3.2 Transaction History Mismatches
**Location**: `transaction_dto.rs` vs `openapi.yaml` `GET /wallet/history`
- **OpenAPI Spec**: Expects `tx_type`, `tx_hash`, `status`, `created_at`.
- **Backend/Frontend**: Uses `movement_type`, `blockchain_hash`, `process_status`, `timestamp`. Additionally includes `fee_deducted: f64`.
- **Impact & Recommendation**: Critical parsing failure if frontend relies on OpenAPI generator. The Rust DTOs have correctly applied the obfuscation/domain-driven names, so **update OpenAPI** to match `TxHistoryItemRes` from `transaction_dto.rs`.

### 3.3 Escrow DTO Mismatches
**Location**: `escrow_dto.rs` vs `openapi.yaml` (`SafeTradeStatusRes`)
- **OpenAPI Spec**: Includes `trade_uid`, `item_uid`, `locked_funds`, `step`, `auto_finalize_deadline`.
- **Backend/Frontend**: Additionally includes `buyer_uid`, `seller_uid`, `created_at`.
- **Impact & Recommendation**: Frontend needs `buyer_uid` and `seller_uid` to differentiate views for buyer vs seller. **Update OpenAPI** to include these fields.

## 4. Data Type, Nullability & Enum Mismatches

| Field | OpenAPI Type | Backend Type | Frontend Type | Issue Description |
|---|---|---|---|---|
| `contact_email` | `string` (Required) | `Option<String>` | `Option<String>` | `SignUpReq` in Rust allows null email, but OpenAPI marks it required. |
| `contact_phone` | `string` (Required) | `Option<String>` | `Option<String>` | `SignUpReq` in Rust allows null phone, but OpenAPI marks it required. |
| `bio` | *Missing* | `Option<String>` | `Option<String>` | `UserProfileRes` returns bio in Rust, but missing in OpenAPI. |
| `ItemState` | `string` enum | `ItemState` Enum | `ItemState` Enum | Types match perfectly, but OpenAPI doesn't account for missing/optional fields. |

## 5. Auth & Header Mechanisms
- **Bearer Token**: Both the specification and implementation agree on the `Bearer <token>` format via the `Authorization` header.
- **Error Responses**: The OpenAPI spec lacks standardized error response definitions (e.g., `401 Unauthorized`, `400 Bad Request` schema). The backend currently maps errors to generic strings which breaks structured error handling on the frontend. A global API Error DTO should be introduced.

## 6. Actionable Modification Plan (Phase 4 Tasks)

### 🔴 High Priority: OpenAPI Spec Fixes
1. **Update `GET /products` Response:** Add `ItemSummaryRes` schema to components and set it as the return type for the array.
2. **Update `GET /wallet/history` Response:** Align fields with `TxHistoryItemRes` (`movement_type`, `fee_deducted`, `blockchain_hash`, `process_status`, `timestamp`).
3. **Update `SafeTradeStatusRes`:** Add `buyer_uid`, `seller_uid`, and `created_at` to the schema.
4. **Update `CreateItemReq`:** Add `image_urls` as an array of strings.
5. **Standardize Auth Optionality:** Modify `SignUpReq` to make `contact_email` and `contact_phone` optional.

### 🟡 Medium Priority: Backend Adjustments
1. **Standardized Error DTO:** Introduce an `ApiErrorRes` struct in `shared/dto/error_dto.rs` to replace generic `Result<T, String>`.
2. **Missing Escrow Endpoint:** Implement `GET /escrow/{trade_uid}` in `escrow_handler.rs` as it's critical for fetching current trade states, despite being missing in OpenAPI.

### 🟢 Low Priority: Frontend Adjustments
1. **Error Handling Mapping:** Update `Action` and `Resource` closures to parse `ApiErrorRes` instead of plain strings once the backend implements it.
2. **Strict Props Assignment:** Clean up remaining `#allow(unused)` imports that were resolved during Phase 3.
