// crates/frontend/src/pages/admin.rs
// URL: /admin
// [접근 제한] is_admin Signal이 false이면 접근 차단

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::admin_dto::ForceSettleReq;

#[component]
pub fn AdminPage() -> impl IntoView {
    // todo!("AuthStore에서 is_admin 확인 → false면 redirect('/')")
    view! {
        <Title text="관리자 어드민"/>
        <main class="admin-page">
            <h1>"관리자 패널"</h1>
            <section class="stats">
                // todo!("GET /v1/admin/stats → PlatformStatsRes 표시")
                <p>"플랫폼 통계 로딩 중..."</p>
            </section>
            <section class="disputes">
                <h2>"분쟁 중재"</h2>
                // todo!("분쟁 목록 + 강제 정산 버튼 → POST /v1/admin/escrow/force-settle")
            </section>
        </main>
    }
}
