// crates/frontend/src/components/input/category_select.rs
// 목적: 카테고리 드롭다운 선택 — design.md 기준 카테고리 목록

use leptos::prelude::*;

const CATEGORIES: &[(&str, &str)] = &[
    ("",            "카테고리 선택"),
    ("electronics", "전자기기"),
    ("clothing",    "의류/패션"),
    ("books",       "도서/문구"),
    ("furniture",   "가구/인테리어"),
    ("sports",      "스포츠/레저"),
    ("beauty",      "뷰티/미용"),
    ("food",        "식품"),
    ("kids",        "유아동"),
    ("etc",         "기타"),
];

#[component]
pub fn CategorySelect(signal: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="form-field">
            <label class="form-label" for="category-select">"카테고리"</label>
            <select
                id="category-select"
                class="form-select"
                prop:value=move || signal.get()
                on:change=move |ev| signal.set(event_target_value(&ev))
            >
                {CATEGORIES.iter().map(|(value, label)| {
                    let v = *value;
                    view! {
                        <option value=v selected=move || signal.get() == v>
                            {*label}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
