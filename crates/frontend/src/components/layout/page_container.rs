// crates/frontend/src/components/layout/page_container.rs
// 목적: 페이지 공통 래퍼 — <Title/> + max-width + 상하 패딩

use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn PageContainer(
    /// 브라우저 탭 제목 및 h1에 사용할 페이지 제목
    #[prop(into)]
    title: String,
    /// 자식 콘텐츠
    children: Children,
) -> impl IntoView {
    let page_title = format!("{} | ETH 중고마켓", title);

    view! {
        <Title text=page_title/>
        <div class="w-full max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-6 md:py-10">
            {children()}
        </div>
    }
}
