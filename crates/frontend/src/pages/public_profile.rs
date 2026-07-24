// crates/frontend/src/pages/public_profile.rs
// URL: /users/:user_uid
// 목적: 공개 사용자 프로필 페이지 — 가입일, ID, 등록한 상품 목록

use crate::components::{
    display::{EmptyState, ProductCard, UserTrustIndicator},
    layout::PageContainer,
};
use crate::models::{product_model::create_products_resource, user_model::fetch_public_profile};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use shared::dto::product_dto::ProductSearchQuery;

#[component]
pub fn PublicProfilePage() -> impl IntoView {
    let params = use_params_map();
    let user_uid = move || params.with(|p| p.get("user_uid").unwrap_or_default());

    let profile_res = LocalResource::new(move || {
        let uid = user_uid();
        async move {
            if uid.is_empty() {
                return Err("유효하지 않은 사용자 ID입니다.".to_string());
            }
            fetch_public_profile(uid).await
        }
    });

    let products_res = create_products_resource(move || {
        let uid = user_uid();
        ProductSearchQuery {
            keyword: None,
            group_category: None,
            item_tag: None,
            owner_uid: if uid.is_empty() { None } else { Some(uid) },
            page: Some(1),
            page_size: Some(50), // Default pagination size for profile view
        }
    });

    view! {
        <Title text="프로필"/>
        <PageContainer title="사용자 프로필">
            <div class="flex flex-col gap-8 max-w-4xl mx-auto w-full">
                // Profile Section
                <section class="bg-white p-6 rounded-2xl shadow-sm border border-slate-100 flex flex-col gap-4">
                    <Suspense fallback=move || view! { <p class="text-slate-400">"프로필 정보를 불러오는 중..."</p> }>
                        {move || {
                            profile_res.get().map(|res| {
                                match &*res {
                                    Ok(profile) => {
                                        let profile_clone = profile.clone();
                                        let joined_date = profile_clone.joined_at.split('T').next().unwrap_or(&profile_clone.joined_at).to_string();
                                        let display_name = profile_clone.display_name.clone();
                                        let user_uid = profile_clone.user_uid.clone();
                                        let reliability_index = profile_clone.reliability_index;
                                        let bio1 = profile_clone.bio.clone();
                                        let bio2 = profile_clone.bio.clone();
                                        view! {
                                            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                                                <div>
                                                    <h2 class="text-2xl font-bold text-slate-900">{display_name}</h2>
                                                    <p class="text-slate-500 font-mono mt-1 text-sm">{format!("ID: {}", user_uid)}</p>
                                                    <p class="text-slate-500 text-sm mt-1">{format!("가입일: {}", joined_date)}</p>
                                                </div>
                                                <div class="bg-slate-50 p-4 rounded-xl border border-slate-100">
                                                    <UserTrustIndicator reliability_index=reliability_index />
                                                </div>
                                            </div>
                                            <Show when=move || bio1.is_some()>
                                                <div class="mt-4 pt-4 border-t border-slate-100 text-slate-700 whitespace-pre-wrap">
                                                    {bio2.clone().unwrap_or_default()}
                                                </div>
                                            </Show>
                                        }.into_any()
                                    }
                                    Err(e) => {
                                        view! {
                                            <div class="p-4 rounded-xl bg-red-50 text-red-600 border border-red-100">
                                                {format!("프로필을 불러오지 못했습니다: {}", e)}
                                            </div>
                                        }.into_any()
                                    }
                                }
                            })
                        }}
                    </Suspense>
                </section>

                // Products Section
                <section>
                    <h3 class="text-xl font-bold text-slate-900 mb-6">"등록한 상품"</h3>
                    <Suspense fallback=move || view! {
                        <div class="flex items-center justify-center py-10">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-600"></div>
                        </div>
                    }>
                        {move || {
                            products_res.get().map(|res| {
                                match &*res {
                                    Ok(items) => {
                                        let items = items.clone();
                                        if items.is_empty() {
                                            view! {
                                                <div class="py-10 bg-white rounded-2xl border border-slate-100">
                                                    <EmptyState
                                                        icon="📦"
                                                        message="등록한 상품이 없습니다."
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                                                    {items.into_iter().map(|item| view! {
                                                        <ProductCard item=item />
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                    Err(err) => {
                                        view! {
                                            <div class="p-4 rounded-xl bg-red-50 text-red-600 border border-red-100 text-center">
                                                {format!("상품을 불러오지 못했습니다: {}", err)}
                                            </div>
                                        }.into_any()
                                    }
                                }
                            })
                        }}
                    </Suspense>
                </section>
            </div>
        </PageContainer>
    }
}
