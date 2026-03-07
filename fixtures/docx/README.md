# DOCX Review Set

Phase 1 DOCX review fixture set.

## Files
- `docx_plain_text.docx`
  - plain text
  - basic page open and paragraph flow
- `docx_styles_lists.docx`
  - styles
  - bullet and nested list parsing
- `docx_tables_images.docx`
  - tables
  - images
  - search and block placement review
- `docx_multi_section.docx`
  - multi-section
  - header/footer and section break review

## Related encrypted fixture
- `../encrypted/docx_password_stub.docx`
  - password-required detection stub

## Review rule
- Keep this set small and stable.
- If a DOCX regression is fixed, add one regression copy under `fixtures/regression/`.
