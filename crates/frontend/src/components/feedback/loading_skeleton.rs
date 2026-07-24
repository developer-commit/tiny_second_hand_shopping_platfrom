// crates/frontend/src/components/feedback/loading_skeleton.rs
// 목적: 데이터 로딩 중 스켈레톤 UI — CSS shimmer 애니메이션
// variant에 따라 Card / List / Detail 형태로 렌더링합니다.

use leptos::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum SkeletonVariant {
    Card,
    List,
    Detail,
}

#[component]
pub fn LoadingSkeleton(
    #[prop(default = SkeletonVariant::Card)]
    variant: SkeletonVariant,
    /// 반복 횟수 (List 변형에서 행 수)
    #[prop(default = 3_usize)]
    count: usize,
) -> impl IntoView {
    match variant {
        SkeletonVariant::Card => view! {
            <div class="skeleton-grid">
                {(0..count).map(|_| view! {
                    <div class="skeleton-card">
                        <div class="skeleton skeleton-card-img"></div>
                        <div class="skeleton-card-body">
                            <div class="skeleton skeleton-text skeleton-text-lg"></div>
                            <div class="skeleton skeleton-text skeleton-text-sm"></div>
                            <div class="skeleton skeleton-text skeleton-text-xs"></div>
                        </div>
                    </div>
                }).collect_view()}
            </div>
        }.into_any(),

        SkeletonVariant::List => view! {
            <div class="skeleton-list">
                {(0..count).map(|_| view! {
                    <div class="skeleton-list-row">
                        <div class="skeleton skeleton-avatar"></div>
                        <div class="skeleton-list-text">
                            <div class="skeleton skeleton-text skeleton-text-lg"></div>
                            <div class="skeleton skeleton-text skeleton-text-sm"></div>
                        </div>
                    </div>
                }).collect_view()}
            </div>
        }.into_any(),

        SkeletonVariant::Detail => view! {
            <div class="skeleton-detail">
                <div class="skeleton skeleton-detail-img"></div>
                <div class="skeleton skeleton-text skeleton-text-xl" style="margin-top: 16px;"></div>
                <div class="skeleton skeleton-text skeleton-text-lg"></div>
                <div class="skeleton skeleton-text skeleton-text-md"></div>
                <div class="skeleton skeleton-text skeleton-text-md"></div>
                <div class="skeleton skeleton-text skeleton-text-sm"></div>
            </div>
        }.into_any(),
    }
}
