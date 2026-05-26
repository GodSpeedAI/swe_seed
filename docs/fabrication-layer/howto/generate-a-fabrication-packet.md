# Generate A Fabrication Packet

Create a run from a seed and then generate the packet:

```bash
python scripts/fabricate.py new path/to/seed.yaml
python scripts/fabricate.py generate <run_id>
```

The generated packet is written to `.fabricator/runs/<run_id>/generated/` and includes the semantic
chain artifacts, eval checklist, eval spec, proof record, reflection template, and adaptation
decision.

For `single_page_html5_game`, the reference implementation also generates a playable
`.fabricator/runs/<run_id>/prototype/index.html`.
