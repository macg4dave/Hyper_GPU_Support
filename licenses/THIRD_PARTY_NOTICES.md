# Third-party notices

Project-authored Hyper GPU Support source is licensed under the [MIT licence](../LICENSE).
Dependencies and the installer runtime retain their own copyright and licence terms.
Both release packages include the enabled Windows runtime dependency inventory and upstream
licence texts in `licenses/dependencies/`.

## Slint

This application uses [Slint](https://slint.dev). Slint 1.18.1 offers GPL-3.0-only,
Slint Royalty-free 2.0 and Slint commercial licensing alternatives. The upstream
licence texts are retained in `licenses/slint/` and the dependency notices.
The public project/download documentation displays the Slint attribution badge.
Project source licensing alone does not replace the terms of linked components.

The vendored `i-slint-backend-winit` contains a local Windows exposure-region
repainting backport. Its upstream identity and licence notices are preserved.

The application uses Slint's Royalty-free Desktop, Mobile and Web Applications
License option. Preserve its attribution and licence texts when redistributing
the application, including this backport.

## Inno Setup

The Windows installer is built with [Inno Setup](https://jrsoftware.org/).
Its copyright and licence terms are included in `licenses/inno/`.
Build tools and unpacking utilities are not installed with Hyper GPU Support.
