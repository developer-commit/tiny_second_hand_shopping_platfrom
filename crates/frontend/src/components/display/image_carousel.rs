// crates/frontend/src/components/display/image_carousel.rs
// 목적: 상품 이미지 슬라이드 갤러리
// RwSignal<usize>로 현재 인덱스를 관리하고 이전/다음 버튼으로 탐색합니다.

use leptos::prelude::*;

#[component]
pub fn ImageCarousel(urls: Vec<String>) -> impl IntoView {
    let total = urls.len();
    let current = RwSignal::new(0_usize);
    let has_images = total > 0;
    let has_multiple = total > 1;

    let cloned_urls = StoredValue::new(urls);
    let current_url = move || {
        cloned_urls
            .get_value()
            .get(current.get())
            .cloned()
            .unwrap_or_default()
    };

    let on_prev = move |_| {
        current.update(|i| {
            if *i > 0 {
                *i -= 1;
            } else {
                *i = total.saturating_sub(1);
            }
        });
    };
    let on_next = move |_| {
        current.update(|i| {
            *i = (*i + 1) % total.max(1);
        });
    };

    view! {
        <div class="image-carousel">
            <Show
                when=move || has_images
                fallback=|| view! {
                    <div class="carousel-placeholder">
                        <span class="carousel-placeholder-icon">"📷"</span>
                        <span>"이미지 없음"</span>
                    </div>
                }
            >
                // 이미지 표시 영역
                <div class="carousel-frame">
                    <img
                        src=move || current_url()
                        alt=move || format!("상품 이미지 {}/{}", current.get() + 1, total)
                        class="carousel-img"
                    />
                    // 인디케이터
                    <span class="carousel-indicator">
                        {move || format!("{} / {}", current.get() + 1, total)}
                    </span>
                </div>
                // 네비게이션 버튼
                <Show when=move || has_multiple>
                    <button class="carousel-btn carousel-btn-prev" on:click=on_prev aria-label="이전 이미지">
                        "‹"
                    </button>
                    <button class="carousel-btn carousel-btn-next" on:click=on_next aria-label="다음 이미지">
                        "›"
                    </button>
                </Show>
                // 썸네일 도트
                <div class="carousel-dots">
                    {(0..total).map(|i| {
                        let dot_class = move || {
                            if current.get() == i { "carousel-dot carousel-dot-active" } else { "carousel-dot" }
                        };
                        view! {
                            <button
                                class=dot_class
                                on:click=move |_| current.set(i)
                                aria-label=format!("이미지 {}", i + 1)
                            ></button>
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>
    }
}
