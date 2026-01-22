# Boomaga-IPP
Boomaga-IPP (BOOklet MAnager - IPP) is a modern (as of 2026) reimplementation
of the Boomaga virtual printer system, written entirely in Rust. The system
provides print preview and booklet printing capabilities using IPP Everywhere
protocol (rather than CUPS backend drivers). Boomaga-IPP is designed specifically
for Wayland environments and managed by systemd. The system consists of an
IPP Everywhere service and a Rust-based GUI application that allows users to
preview documents before printing and create various print layouts, including
booklets and multi-page arrangements.
