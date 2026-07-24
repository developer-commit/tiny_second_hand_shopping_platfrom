// crates/frontend/src/components/input/star_rating.rs
// 목적: 1~5점 별점 입력/표시 컴포넌트
// readonly=true이면 버튼 대신 텍스트로 표시합니다.

use leptos::prelude::*;

#[component]
pub fn StarRating(
    score: RwSignal<i32>,
    #[prop(default = false)]
    readonly: bool,
) -> impl IntoView {
    view! {
        <div class="star-rating" role=if readonly { "img" } else { "group" } aria-label=move || format!("별점 {}점", score.get())>
            {(1..=5_i32).map(|i| {
                let is_filled = move || score.get() >= i;
                if readonly {
                    view! {
                        <span class=move || if is_filled() { "star star-filled" } else { "star" } aria-hidden="true">
                            {move || if is_filled() { "★" } else { "☆" }}
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <button
                            type="button"
                            class=move || if is_filled() { "star star-filled" } else { "star" }
                            on:click=move |_| score.set(i)
                            aria-label=format!("{}점", i)
                            aria-pressed=move || (score.get() >= i).to_string()
                        >
                            {move || if is_filled() { "★" } else { "☆" }}
                        </button>
                    }.into_any()
                }
            }).collect_view()}
        </div>
    }
}
