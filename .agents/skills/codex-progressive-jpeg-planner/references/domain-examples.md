# Purpose-Solving Slice Examples

| Domain | Smallest honest product slice | Not sufficient by itself |
| --- | --- | --- |
| Software product | A real user completes one narrow workflow safely through the intended production path and can continue or return. | API/UI skeleton, compiled screen, mocked success, internal event. |
| Game actor | A player triggers, perceives, and understands one behavior that contributes to the real gameplay loop. Planned seams may debug, but the player outcome cannot depend on debug. | Spawn plus debug log, component inventory, animation hook without gameplay feedback. |
| Site | The target visitor understands the offer, sees honest proof, and completes a real next action such as contact, signup, booking, purchase, or use. | Shell, routes, contacts, or coming-soon copy unless lead capture/announcement is the actual accepted product purpose. |
| Book or text | The target reader receives one coherent useful argument, answer, procedure, or transformation in a complete condensed reading path. | TOC, synopsis, chapter slots, sample voice, disconnected draft. |
| Internal tool | The real operator completes one actual task faster, safer, or with fewer errors in the workflow where the result is used. | Dashboard shell, unused report, synthetic data, export without downstream action. |
| Technical module | A downstream developer completes one supported integration safely through the public contract and obtains a useful result. | Empty API, no-op response, types without a working supported path. |

Use manual or concierge fulfillment only when it genuinely delivers the same
user outcome and is identified as the current operating mechanism. Do not hide
manual fulfillment behind fake automation.

For every domain, ask:

1. Who enters, from where, and why?
2. What meaningful action can they complete now?
3. What result do they perceive or use?
4. How do they continue, exit, or return?
5. What observation would falsify the claimed value?
