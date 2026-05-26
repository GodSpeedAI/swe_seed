# Create A Product Seed

Keep the seed bounded. It should describe one product idea, one user situation, one desired
outcome, and a short list of constraints, non-goals, success conditions, and proof expectations.

Required seed fields:

- `run_id`
- `product_type`
- `user`
- `situation`
- `desired_outcome`
- `prototype_goal`

The reference implementation accepts JSON or simple YAML objects. A minimal example lives in
`tests/validate-harness.sh` and is used as the fabrication smoke run.
