// crates/frontend/src/components/input/tag_input.rs
// 목적: 태그 칩 입력 — Enter로 추가, × 클릭으로 삭제

use leptos::prelude::*;

#[component]
pub fn TagInput(tags: RwSignal<Vec<String>>) -> impl IntoView {
    let input_val = RwSignal::new(String::new());

    let add_tag = move || {
        let tag = input_val.get().trim().to_string();
        if !tag.is_empty() && !tags.get().contains(&tag) {
            tags.update(|v| v.push(tag));
            input_val.set(String::new());
        }
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" || ev.key() == "," {
            ev.prevent_default();
            add_tag();
        }
    };

    view! {
        <div class="tag-input-wrapper">
            <label class="form-label">"태그"</label>
            <div class="tag-chip-list">
                // 현재 태그들
                {move || tags.get().into_iter().enumerate().map(|(i, tag)| {
                    let tag_label = tag.clone();
                    view! {
                        <span class="tag-chip">
                            {"#"}{tag_label}
                            <button
                                type="button"
                                class="tag-chip-remove"
                                on:click=move |_| tags.update(|v| { v.remove(i); })
                                aria-label=format!("태그 {} 삭제", tag.clone())
                            >
                                "×"
                            </button>
                        </span>
                    }
                }).collect_view()}

                // 태그 입력 필드
                <input
                    type="text"
                    class="tag-input"
                    placeholder="태그 입력 후 Enter"
                    prop:value=move || input_val.get()
                    on:input=move |ev| input_val.set(event_target_value(&ev))
                    on:keydown=on_keydown
                    aria-label="새 태그 입력"
                />
            </div>
            <p class="form-hint">"Enter 또는 쉼표(,)로 태그를 추가합니다."</p>
        </div>
    }
}
