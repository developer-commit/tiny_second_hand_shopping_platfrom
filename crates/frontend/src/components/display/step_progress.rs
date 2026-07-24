// crates/frontend/src/components/display/step_progress.rs
// 목적: 에스크로 진행 단계 가로 스텝퍼
// steps: 단계 레이블 Vec, current: 0-based 현재 단계 인덱스

use leptos::prelude::*;

#[component]
pub fn StepProgress(
    steps: Vec<String>,
    current: usize,
) -> impl IntoView {
    let total = steps.len();

    view! {
        <div class="step-progress" aria-label="진행 단계">
            {steps.into_iter().enumerate().map(|(i, label)| {
                let is_done    = i < current;
                let is_current = i == current;
                let is_last    = i == total - 1;

                let dot_class = if is_done || is_current {
                    "step-dot step-dot-active"
                } else {
                    "step-dot"
                };
                let line_class = if is_done {
                    "step-line step-line-done"
                } else {
                    "step-line"
                };

                view! {
                    <div class="step-item">
                        <div class=dot_class>
                            <Show
                                when=move || is_done
                                fallback=move || view! { <span class="step-dot-num">{i + 1}</span> }
                            >
                                <span class="step-dot-check">"✓"</span>
                            </Show>
                        </div>
                        <span class=if is_current { "step-label step-label-current" } else { "step-label" }>
                            {label}
                        </span>
                        <Show when=move || !is_last>
                            <div class=line_class></div>
                        </Show>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
