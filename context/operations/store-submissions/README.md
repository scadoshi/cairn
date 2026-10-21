# Store submission copy

One release's "What's New" text, in one place, ready to paste. Each version gets a directory with a `whats_new.md` holding the copy in a fenced block, so it can be copied without picking it out of a larger document. Same shape as zwipe's directory, so the two runbooks read the same.

## The 500-character rule

The App Store allows 4,000 characters. Write to 500 anyway, which is Play's limit, so the text would go to both stores unchanged if an Android build ever ships, and so the notes stay short. Every file states its own character count.

## Writing a new one

Start from the version's heading in [`../../progress/changelog.md`](../../progress/changelog.md), which is already in the right voice.

- Compress to fit 500, name the actual controls, and skip anything a user cannot see.
- Each line is a phrase: no period at the end, no examples.

Then paste the text into App Store Connect at the submit step of [`../ios/app-store/submission/publish.md`](../ios/app-store/submission/publish.md). The rest of the listing copy lives in [`../ios/app-store/submission/form_fields.md`](../ios/app-store/submission/form_fields.md).

## Releases

| Version | Characters |
|---------|-----------|
| [1.1.0](1.1.0/whats_new.md) | 485 |
| [1.0.0](1.0.0/whats_new.md) | 319 |
