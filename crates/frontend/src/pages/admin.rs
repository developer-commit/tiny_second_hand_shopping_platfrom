// crates/frontend/src/pages/admin.rs
// URL: /admin
// [접근 제한] is_admin Signal이 false이면 접근 차단

use crate::components::{
    display::StatCard,
    feedback::button::{AppButton, ButtonVariant},
    input::form_input::FormInput,
    layout::PageContainer,
};
use crate::models::auth_model::AuthStore;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use shared::dto::admin_dto::{
    AdminReportListRes, BanUserReq, ForceSettleReq, ForceSettleTarget, HideProductReq,
    PlatformStatsRes,
};

async fn fetch_admin_stats(token: &str) -> Result<PlatformStatsRes, String> {
    let url = "/v1/admin/stats";
    let res = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("통계 로드 실패".into());
    }
    res.json().await.map_err(|e| e.to_string())
}

async fn fetch_admin_reports(token: &str) -> Result<AdminReportListRes, String> {
    let url = "/v1/admin/reports";
    let res = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("신고 목록 로드 실패".into());
    }
    res.json().await.map_err(|e| e.to_string())
}

async fn force_settle(token: &str, req: &ForceSettleReq) -> Result<(), String> {
    let url = "/v1/admin/escrow/force-settle";
    let res = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("강제 정산 실패".into());
    }
    Ok(())
}

async fn ban_user_api(token: &str, user_uid: &str, req: &BanUserReq) -> Result<(), String> {
    let url = format!("/v1/admin/users/{}/ban", user_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("사용자 밴 실패".into());
    }
    Ok(())
}

async fn hide_product_api(token: &str, item_uid: &str, req: &HideProductReq) -> Result<(), String> {
    let url = format!("/v1/admin/products/{}/hide", item_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("상품 숨김 실패".into());
    }
    Ok(())
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let navigate = use_navigate();

    // Redirect if not admin
    Effect::new(move |_| {
        let is_admin = auth_store
            .current_user
            .get()
            .map(|u| u.role == "admin")
            .unwrap_or(false);
        if !is_admin {
            navigate("/", Default::default());
        }
    });

    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let stats_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() {
                return Err("로그인이 필요합니다.".to_string());
            }
            fetch_admin_stats(&t).await
        }
    });

    let trade_uid_input = RwSignal::new(String::new());
    let reason_input = RwSignal::new(String::new());
    let force_settle_action = Action::new_local(move |req: &ForceSettleReq| {
        let t = token.get();
        let req_clone = req.clone();
        async move { force_settle(&t, &req_clone).await }
    });

    let on_force_settle = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = ForceSettleReq {
            trade_uid: trade_uid_input.get(),
            settle_to: ForceSettleTarget::Buyer, // Using Buyer as default for UI simplicity
            reason: reason_input.get(),
        };
        force_settle_action.dispatch(req);
    };

    let reports_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() {
                return Err("로그인이 필요합니다.".to_string());
            }
            fetch_admin_reports(&t).await
        }
    });

    let ban_user_uid = RwSignal::new(String::new());
    let ban_reason = RwSignal::new(String::new());
    let ban_action = Action::new_local(move |req: &(String, BanUserReq)| {
        let t = token.get();
        let req_clone = req.clone();
        async move { ban_user_api(&t, &req_clone.0, &req_clone.1).await }
    });

    let on_ban_user = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = BanUserReq {
            reason: ban_reason.get(),
        };
        ban_action.dispatch((ban_user_uid.get(), req));
    };

    let hide_item_uid = RwSignal::new(String::new());
    let hide_reason = RwSignal::new(String::new());
    let hide_action = Action::new_local(move |req: &(String, HideProductReq)| {
        let t = token.get();
        let req_clone = req.clone();
        async move { hide_product_api(&t, &req_clone.0, &req_clone.1).await }
    });

    let on_hide_product = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = HideProductReq {
            reason: hide_reason.get(),
        };
        hide_action.dispatch((hide_item_uid.get(), req));
    };

    view! {
        <Title text="관리자 어드민"/>
        <PageContainer title="관리자 어드민">
            <div class="admin-page" style="display: flex; flex-direction: column; gap: 2rem; padding-top: 2rem;">
                <h1 style="font-size: 1.5rem; font-weight: bold;">"관리자 패널"</h1>

                <section class="stats">
                    <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"플랫폼 통계"</h2>
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 1rem;">
                        <Suspense fallback=move || view! { <p>"통계 불러오는 중..."</p> }>
                            {move || stats_res.get().map(|res| match &*res {
                                Ok(stats) => view! {
                                    <StatCard label="총 누적 수수료" value=format!("{} ETH", stats.total_accumulated_fees) icon="📈" />
                                    <StatCard label="총 사용자 수" value=format!("{} 명", stats.total_users) icon="👥" />
                                    <StatCard label="활성 상품" value=format!("{} 개", stats.active_listings) icon="📦" />
                                    <StatCard label="진행중인 에스크로" value=format!("{} 건", stats.active_escrows) icon="🔒" />
                                    <StatCard label="대기중인 분쟁" value=format!("{} 건", stats.pending_disputes) icon="⚠️" />
                                    <StatCard label="대기중인 신고" value=format!("{} 건", stats.pending_reports) icon="🚨" />
                                }.into_any(),
                                Err(e) => view! { <p style="color: red;">{e.clone()}</p> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </section>

                <section class="disputes">
                    <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"운영 관리"</h2>
                    <div style="display: flex; flex-direction: column; gap: 1.5rem;">
                        <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                            <h3 style="margin-bottom: 1rem;">"강제 정산"</h3>
                            <form on:submit=on_force_settle style="display: flex; flex-direction: column; gap: 1rem; max-width: 400px;">
                                <FormInput label="거래 ID (trade_uid)" placeholder="정산할 거래 ID" input_type="text" signal=trade_uid_input error=Signal::derive(|| None) />
                                <FormInput label="정산 사유" placeholder="강제 정산 사유 입력" input_type="text" signal=reason_input error=Signal::derive(|| None) />
                                <AppButton variant=ButtonVariant::Danger loading=force_settle_action.pending() disabled=force_settle_action.pending() button_type="submit">
                                    "강제 정산 실행"
                                </AppButton>
                            </form>
                        </div>

                        <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                            <h3 style="margin-bottom: 1rem;">"사용자 제재 (밴)"</h3>
                            <form on:submit=on_ban_user style="display: flex; flex-direction: column; gap: 1rem; max-width: 400px;">
                                <FormInput label="사용자 ID (user_uid)" placeholder="제재할 사용자 ID" input_type="text" signal=ban_user_uid error=Signal::derive(|| None) />
                                <FormInput label="제재 사유" placeholder="제재 사유 입력" input_type="text" signal=ban_reason error=Signal::derive(|| None) />
                                <AppButton variant=ButtonVariant::Danger loading=ban_action.pending() disabled=ban_action.pending() button_type="submit">
                                    "사용자 밴 실행"
                                </AppButton>
                            </form>
                        </div>

                        <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                            <h3 style="margin-bottom: 1rem;">"상품 강제 숨김"</h3>
                            <form on:submit=on_hide_product style="display: flex; flex-direction: column; gap: 1rem; max-width: 400px;">
                                <FormInput label="상품 ID (item_uid)" placeholder="숨길 상품 ID" input_type="text" signal=hide_item_uid error=Signal::derive(|| None) />
                                <FormInput label="숨김 사유" placeholder="숨김 사유 입력" input_type="text" signal=hide_reason error=Signal::derive(|| None) />
                                <AppButton variant=ButtonVariant::Danger loading=hide_action.pending() disabled=hide_action.pending() button_type="submit">
                                    "상품 숨김 실행"
                                </AppButton>
                            </form>
                        </div>
                    </div>
                </section>

                <section class="reports">
                    <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"신고 목록"</h2>
                    <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                        <Suspense fallback=move || view! { <p>"신고 목록 불러오는 중..."</p> }>
                            {move || reports_res.get().map(|res| match &*res {
                                Ok(reports_res) => {
                                    if reports_res.reports.is_empty() {
                                        view! { <p>"대기중인 신고가 없습니다."</p> }.into_any()
                                    } else {
                                        view! {
                                            <table style="width: 100%; border-collapse: collapse; text-align: left;">
                                                <thead>
                                                    <tr style="border-bottom: 1px solid var(--border-color); padding-bottom: 0.5rem;">
                                                        <th style="padding: 0.5rem;">"신고 대상"</th>
                                                        <th style="padding: 0.5rem;">"신고자"</th>
                                                        <th style="padding: 0.5rem;">"사유"</th>
                                                        <th style="padding: 0.5rem;">"상태"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {reports_res.reports.iter().map(|r| view! {
                                                        <tr style="border-bottom: 1px solid var(--border-color);">
                                                            <td style="padding: 0.5rem;">{r.target_item_uid.clone()}</td>
                                                            <td style="padding: 0.5rem;">{r.reporter_uid.clone()}</td>
                                                            <td style="padding: 0.5rem;">{r.cause.clone()}</td>
                                                            <td style="padding: 0.5rem;">{r.status.clone()}</td>
                                                        </tr>
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                }
                                Err(e) => view! { <p style="color: red;">{e.clone()}</p> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </section>
            </div>
        </PageContainer>
    }
}
