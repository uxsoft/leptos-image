use crate::optimizer::*;

use leptos::prelude::*;
use leptos_meta::Link;

/**
 */

/// Image component for rendering optimized static images.
/// Images MUST be static. Will not work with dynamic images.
#[component]
pub fn Image(
    /// Image source. Should be path relative to root.
    #[prop(into)]
    src: String,
    /// Resize image height, but will still maintain the same aspect ratio.
    height: u32,
    /// Resize image width, but will still maintain the same aspect ratio.
    width: u32,
    /// Image quality. 0-100.
    #[prop(default = 75_u8)]
    quality: u8,
    /// Will add blur image to head if true.
    #[prop(default = false)]
    blur: bool,
    /// Will add preload link to head if true.
    #[prop(default = false)]
    priority: bool,
    /// Lazy load image.
    #[prop(default = true)]
    lazy: bool,
    /// Image alt text.
    #[prop(into, optional)]
    alt: String,
) -> impl IntoView {
    if src.starts_with("http") {
        logging::console_debug_warn("Image component only supports static images.");
        let loading = if lazy { "lazy" } else { "eager" };
        return view! { <img src=src alt=alt loading=loading/> }.into_any();
    }

    let blur_image = {
        CachedImage {
            src: src.clone(),
            option: CachedImageOption::Blur(Blur {
                width: 20,
                height: 20,
                svg_width: 100,
                svg_height: 100,
                sigma: 15,
            }),
        }
    };

    let opt_image = {
        CachedImage {
            src: src.clone(),
            option: CachedImageOption::Resize(Resize {
                quality,
                width,
                height,
            }),
        }
    };

    // Retrieve value from Cache if it exists. Doing this per-image to allow image introspection.
    let resource = crate::use_image_cache_resource();

    let blur_image = StoredValue::new(blur_image);
    let opt_image = StoredValue::new(opt_image);
    let alt = StoredValue::new(alt);

    view! {
        <Suspense fallback=|| ()>
            {move || {
                resource
                    .get()
                    .map(|config| {
                        let images = config.cache;
                        let handler_path = config.api_handler_path;
                        let opt_image = opt_image.get_value().get_url_encoded(&handler_path);
                        if blur {
                            let placeholder_svg = images
                                .iter()
                                .find(|(c, _)| blur_image.with_value(|b| b == c))
                                .map(|c| c.1.clone());
                            let svg = {
                                match placeholder_svg {
                                    Some(svg_data) => SvgImage::InMemory(svg_data),
                                    None =>  SvgImage::Request(
                                        blur_image.get_value().get_url_encoded(&handler_path),
                                    )
                                }
                            };
                            let alt = alt.get_value();
                            view! { <CacheImage lazy svg opt_image alt priority/> }
                                .into_any()
                        } else {
                            let loading = if lazy { "lazy" } else { "eager" };
                            view! {
                                <img
                                    alt=alt.get_value()
                                    decoding="async"
                                    loading=loading
                                    src=opt_image
                                />
                            }
                                .into_any()
                        }
                    })
            }}

        </Suspense>
    }.into_any()
}

enum SvgImage {
    InMemory(String),
    Request(String),
}

#[component]
fn CacheImage(
    svg: SvgImage,
    #[prop(into)] opt_image: String,
    #[prop(into, optional)] alt: String,
    priority: bool,
    lazy: bool,
) -> impl IntoView {
    use base64::{engine::general_purpose, Engine as _};

    let style = {
        let background_image = match svg {
            SvgImage::InMemory(svg_data) => {
                let svg_encoded = general_purpose::STANDARD.encode(svg_data.as_bytes());
                format!("url('data:image/svg+xml;base64,{svg_encoded}')")
            }
            SvgImage::Request(svg_url) => {
                format!("url('{}')", svg_url)
            }
        };
        let style= format!(
        "color:transparent;background-size:cover;background-position:50% 50%;background-repeat:no-repeat;background-image:{background_image};",
        );

        style
    };

    let loading = if lazy { "lazy" } else { "eager" };

    view! {
        {if priority {
            view! { <Link rel="preload" as_="image" href=opt_image.clone()/> }.into_any()
        } else {
            ().into_any()
        }}

        <img
            alt=alt.clone()
            decoding="async"
            loading=loading
            src=opt_image
            style=style
        />
    }
}
