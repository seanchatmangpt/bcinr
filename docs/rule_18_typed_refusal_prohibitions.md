Here is the exact list of actions an unsupported input may *not* trigger, according to Rule 18 ("Typed refusals") in `AGENTS.md`:

No unsupported input may:
* panic;
* silently clamp outside the admitted policy;
* drop a factor;
* fall back to a simpler algorithm;
* mutate partial state;
* return a plausible default.
