/// Admin-only styles. The route owns this sheet, so leaving the admin area
/// removes it from the document.
pub const ADMIN_STYLE: &str = r#"
:root{color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;min-height:100dvh;background:#000;color:#d4d7db;
  font-family:"IBM Plex Mono",ui-monospace,monospace;text-rendering:optimizeLegibility}
.login{display:grid;grid-template-columns:360px 1fr;min-height:100dvh}
.admin-aside{border-right:1px solid #1e2126;padding:3rem 2.25rem;display:flex;flex-direction:column;min-height:0}
.eyebrow{font-size:10px;letter-spacing:.24em;text-transform:uppercase;color:#767d87}
.admin-heading{margin:.7rem 0 0;font-family:"IBM Plex Sans",sans-serif;font-weight:600;font-size:1.5rem;color:#fff}
.lede{margin:.6rem 0 0;font-size:13px;line-height:1.6;color:#8b939d}
.admin-form{margin-top:2.2rem}
.admin-label{display:block;margin-top:1.3rem;font-size:12px;color:#8b939d}
.admin-input{margin-top:.4rem;display:block;width:100%;background:#0b0e11;color:#fff;
  border:1px solid #2b3037;padding:.5rem .65rem;font:inherit;font-size:13px}
.admin-input:focus{outline:none;border-color:#e2a340}
.admin-textarea{margin-top:.4rem;display:block;width:100%;min-height:55vh;background:#0b0e11;color:#fff;
  border:1px solid #2b3037;padding:.6rem .7rem;font:inherit;font-size:13px;line-height:1.65;resize:vertical}
.admin-textarea:focus{outline:none;border-color:#e2a340}
.err{margin-top:1.2rem;font-size:12px;color:#e2a340}
.admin-button{margin-top:1.6rem;width:100%;background:#080a0d;color:#fff;border:1px solid #30363d;
  padding:.55rem;font:inherit;font-size:13px;cursor:pointer}
.admin-button:hover{border-color:#e2a340}
.foot{margin-top:auto;padding-top:2rem;font-size:11px;color:#767d87}
.stage{position:relative;overflow:hidden;display:flex;flex-direction:column;padding:3rem 3.25rem}
.tag{position:absolute;top:1.5rem;right:2rem;font-size:11px;letter-spacing:.18em;
  text-transform:uppercase;color:#2b3037}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:0 3rem}
.grp{font-size:10px;letter-spacing:.2em;text-transform:uppercase;color:#767d87;margin:2rem 0 .8rem}
.grp:first-child{margin-top:0}
.admin-dl{display:grid;grid-template-columns:13ch 1fr;gap:.55rem 2ch;margin:0;font-size:13px}
.admin-dl dt{color:#8b939d}
.admin-dl dd{margin:0;color:#c3c9cf}
.admin-dl dd b{color:#e2a340;font-weight:500}
.pages{margin:.2rem 0 0;font-size:13px}
.pages p{margin:.35rem 0;color:#8b939d}
.pages .n{color:#6b7280;margin-right:1.5ch}
.mark{margin-top:auto;font-family:"IBM Plex Sans",sans-serif;font-weight:600;
  letter-spacing:-.04em;line-height:.9;font-size:clamp(2rem,4vw,3.2rem);color:#0e1116}
.eyebrow a{color:inherit;text-decoration:none}
.eyebrow a:hover{color:#8b939d}
.dash{display:grid;grid-template-columns:320px 1fr;min-height:100dvh}
.nav{list-style:none;margin:1rem 0 0;padding:0}
.nav a{display:flex;align-items:center;gap:1.1ch;padding:.45rem 0;font-size:13px;
  color:#8b939d;text-decoration:none}
.nav a:hover{color:#e2a340}
.nav a:hover .ico{color:#e2a340}
.nav a[aria-current]{color:#fff}
.nav a[aria-current] .ico{color:#8b939d}
.bottom{margin-top:auto;padding-top:2.5rem}
.bottom .admin-button{width:auto;padding:.55rem 1.4rem}
.projects{list-style:none;margin:1.4rem 0 0;padding:0;max-width:720px}
.projects li{border-bottom:1px solid #1e2126}
.projects a{display:block;padding:1.25rem 0;text-decoration:none;color:inherit}
.row{display:flex;justify-content:space-between;align-items:baseline;gap:2ch}
.name{font-size:15px;color:#fff}
.projects a:hover .name{color:#e2a340}
.edit{display:inline-flex;align-items:center;color:#6b7280}
.edit .i{width:14px;height:14px;fill:none;stroke:currentColor;stroke-width:2;
  stroke-linecap:round;stroke-linejoin:round}
.projects a:hover .edit{color:#e2a340}
.name-row{display:flex;align-items:center;gap:1.1ch}
.ico{width:16px;height:16px;flex:none;fill:none;stroke:currentColor;stroke-width:1.75;
  stroke-linecap:round;stroke-linejoin:round;color:#767d87}
.projects a:hover .ico{color:#e2a340}
.sum{margin:.5rem 0 0;font-size:13px;line-height:1.55;color:#8b939d}
.meta{margin:.6rem 0 0;font-size:11px;letter-spacing:.04em;color:#767d87}
.meta b{color:#8b939d;font-weight:400}
.meta .path{color:#6b7280}
.wrap{width:100%;max-width:1200px;margin-inline:auto}
.wrap.narrow{max-width:760px}
.wrap .admin-button{width:auto;padding:.55rem 1.4rem}
.mdlabel{font-size:12px;color:#8b939d;margin:1.3rem 0 0}
.md{display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;margin-top:.5rem}
.md .admin-textarea{min-height:55vh;margin-top:0}
.panelabel{font-size:10px;letter-spacing:.2em;text-transform:uppercase;color:#767d87;margin:0 0 .5rem}
.preview{background:#050607;border:1px solid #2b3037;padding:1rem 1.15rem;overflow:auto;min-height:55vh}
.prose{color:#c3c9cf;font-family:"IBM Plex Sans",sans-serif;font-size:14px;line-height:1.65}
.prose>:first-child{margin-top:0}
.prose h1,.prose h2,.prose h3{color:#fff;font-weight:600;line-height:1.3;margin:1.4em 0 .5em}
.prose h2{font-size:1.1rem}
.prose h3{font-size:1rem}
.prose p{margin:.8em 0}
.prose a{color:#e2a340}
.prose strong{color:#fff}
.prose code{font-family:"IBM Plex Mono",ui-monospace,monospace;font-size:.85em;background:#0b0e11;
  border:1px solid #1e2126;padding:.1em .35em;border-radius:3px}
.prose pre{background:#0b0e11;border:1px solid #1e2126;padding:.9rem;overflow:auto;border-radius:4px}
.prose pre code{background:none;border:none;padding:0}
.prose ul,.prose ol{padding-left:1.4em;margin:.8em 0}
.prose li{margin:.3em 0}
@media (max-width:960px){.md{grid-template-columns:1fr}}
:focus-visible{outline:2px solid #e2a340;outline-offset:2px}
@media (max-width:720px){.login{grid-template-columns:1fr}.stage{display:none}.admin-aside{border-right:none}}
"#;
