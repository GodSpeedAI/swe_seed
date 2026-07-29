//! Fabricator evidence links (spec 0020 §14). Gateway audit/proof ids can be
//! cited by `FabricatorProofRecord.evidence_refs` and/or carried on a
//! `TraceabilityLink` — but only as EVIDENCE. Gateway request routing MUST NOT
//! mutate Fabricator semantic-chain artifacts, and chain validation
//! (`validate_semantic_chain`) must stay intact (spec 0020 §14). This module
//! holds the small citation helpers; the chain itself is never touched here.

use crate::fabricator::artifacts::{FabricatorProofRecord, TraceabilityLink};

/// Cite a gateway audit/proof record id on a Fabricator proof record
/// (spec 0020 §14: "Gateway audit records may become evidence refs in
/// `FabricatorProofRecord`"). Pure append; the chain is validated elsewhere.
pub fn cite_gateway_evidence(proof: &mut FabricatorProofRecord, gateway_ref: &str) {
    if !gateway_ref.trim().is_empty() && !proof.evidence_refs.iter().any(|e| e == gateway_ref) {
        proof.evidence_refs.push(gateway_ref.to_string());
    }
}

/// Build a `TraceabilityLink` connecting a gateway evidence ref to a proof
/// record, using the existing free-form `relation` vocabulary (`"proves"`).
pub fn evidence_link(upstream_id: &str, downstream_id: &str) -> TraceabilityLink {
    TraceabilityLink {
        upstream_id: upstream_id.into(),
        downstream_id: downstream_id.into(),
        relation: "proves".into(),
        waiver_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabricator::artifacts::FabricatorProofRecord;

    fn proof() -> FabricatorProofRecord {
        FabricatorProofRecord {
            id: "p".into(),
            run_id: "r".into(),
            eval_id: "e".into(),
            status: "Pending".into(),
            evidence: vec![],
            created_at: String::new(),
            satisfied_check_ids: vec![],
            linked_eval_result_id: None,
            evidence_refs: vec![],
        }
    }

    #[test]
    fn cite_appends_dedup() {
        let mut p = proof();
        cite_gateway_evidence(&mut p, "gateway-audit-1");
        cite_gateway_evidence(&mut p, "gateway-audit-1"); // dedup
        cite_gateway_evidence(&mut p, "  "); // ignored
        assert_eq!(p.evidence_refs, vec!["gateway-audit-1".to_string()]);
    }

    #[test]
    fn evidence_link_uses_proves_relation() {
        let l = evidence_link("audit-1", "proof-rec");
        assert_eq!(l.relation, "proves");
        assert_eq!(l.upstream_id, "audit-1");
        assert_eq!(l.downstream_id, "proof-rec");
        assert!(l.waiver_id.is_none());
    }
}
