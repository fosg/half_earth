use serde::Serialize;
use super::{ProductionOrder, Priority, planner};
use crate::kinds::{ResourceMap, ByproductMap, FeedstockMap, OutputMap, Output, Feedstock};

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub enum ProcessFeature {
    UsesPesticides,
    UsesSynFertilizer,
    UsesLivestock,
    IsIntermittent,
    IsNuclear,
    IsSolar,
    IsCCS,
    IsCombustion,
    IsFossil,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum ProcessStatus {
    Neutral,
    Banned,
    Promoted
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum ProcessChange {
    Neutral,
    Expanding,
    Contracting
}

// TODO use this for labor?
#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum ProcessIntensity {
    Low,
    Medium,
    High
}

#[derive(Debug, Serialize, Clone)]
pub struct Process {
    pub id: usize,
    pub name: &'static str,
    pub mix_share: f32,
    pub limit: Option<f32>,
    pub output: Output,

    // Should start at 1.
    pub output_modifier: f32,

    pub resources: ResourceMap<f32>,
    pub byproducts: ByproductMap<f32>,
    pub feedstock: (Feedstock, f32),

    pub features: Vec<ProcessFeature>,

    // If the player has unlocked and/or banned/promoted
    // this process.
    pub locked: bool,
    pub status: ProcessStatus,

    pub change: ProcessChange,

    pub supporters: Vec<usize>,
    pub opposers: Vec<usize>,
}

impl Process {
    /// Generates production orders based on the provided demand
    /// and this sector's process mix.
    pub fn production_order(&self, demand: &OutputMap<f32>) -> ProductionOrder {
        ProductionOrder {
            process: &self,
            amount: demand[self.output] * self.mix_share,
        }
    }

    pub fn is_banned(&self) -> bool {
        self.status == ProcessStatus::Banned
    }

    pub fn is_promoted(&self) -> bool {
        self.status == ProcessStatus::Promoted
    }
}

/// Update this process mixes to better match
/// the demand and resource weights (by scarcity).
/// This mix adjustment happens at a speed of `sector.momentum`.
pub fn update_mixes(
    processes: &mut [Process],
    demand: &OutputMap<f32>,
    resource_weights: &ResourceMap<f32>,
    feedstock_weights: &FeedstockMap<f32>,
    priority: &Priority) {
    let mut processes_by_output: OutputMap<Vec<&mut Process>> = OutputMap::default();
    for process in processes {
        processes_by_output[process.output].push(process);
    }

    for (output, d) in demand.items() {
        let mut mix: Vec<f32> = processes_by_output[output].iter().map(|p| p.mix_share).collect();

        mix = planner::calculate_mix(mix, *d, &processes_by_output[output], &resource_weights, &feedstock_weights, &priority);
        for (p, share) in processes_by_output[output].iter_mut().zip(&mix) {
            if p.mix_share != *share {
                if p.mix_share < *share {
                    p.change = ProcessChange::Expanding;
                } else {
                    p.change = ProcessChange::Contracting;
                }
                p.mix_share = *share;
            } else {
                p.change = ProcessChange::Neutral;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::planner::Priority;
    use float_cmp::approx_eq;

    fn gen_processes() -> Vec<Process> {
        vec![Process {
            id: 0,
            name: "Test Process A",
            mix_share: 0.5,
            limit: None,
            change: ProcessChange::Neutral,
            output: Output::Fuel,
            output_modifier: 1.,
            resources: resources!(water: 10.),
            byproducts: byproducts!(),
            feedstock: (Feedstock::Oil, 1.),
            features: vec![],
            locked: false,
            status: ProcessStatus::Neutral,
            opposers: vec![],
            supporters: vec![],
        }, Process {
            id: 1,
            name: "Test Process B",
            mix_share: 0.5,
            limit: None,
            change: ProcessChange::Neutral,
            output: Output::Fuel,
            output_modifier: 1.,
            resources: resources!(water: 10.),
            byproducts: byproducts!(),
            feedstock: (Feedstock::Oil, 1.),
            features: vec![],
            locked: false,
            status: ProcessStatus::Neutral,
            opposers: vec![],
            supporters: vec![],
        }, Process {
            id: 2,
            name: "Test Process C",
            mix_share: 1.0,
            limit: None,
            change: ProcessChange::Neutral,
            output: Output::Electricity,
            output_modifier: 1.,
            resources: resources!(water: 2.),
            byproducts: byproducts!(),
            feedstock: (Feedstock::Oil, 1.),
            features: vec![],
            locked: false,
            status: ProcessStatus::Neutral,
            opposers: vec![],
            supporters: vec![],
        }]
    }

    #[test]
    fn test_update_mix_share_resources() {
        let priority = Priority::Scarcity;
        let mut processes = gen_processes();
        processes[1].resources = resources!(water: 2.);
        let demand = outputs!(fuel: 100.);
        let resource_weights = resources!(water: 100.);
        let feedstock_weights = feedstocks!();
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Less water intensive process should be favored
        assert!(processes[0].mix_share < processes[1].mix_share);

        assert_eq!(processes[0].change, ProcessChange::Contracting);
        assert_eq!(processes[1].change, ProcessChange::Expanding);

        // Should be normalized
        assert_eq!(processes[0].mix_share + processes[1].mix_share, 1.0);

        // Unrelated process should be unaffected
        assert_eq!(processes[2].mix_share, 1.0);
    }

    #[test]
    fn test_update_mix_share_banned() {
        let priority = Priority::Scarcity;
        let mut processes = gen_processes();
        processes[0].status = ProcessStatus::Banned;
        let demand = outputs!(fuel: 100.);
        let resource_weights = resources!();
        let feedstock_weights = feedstocks!();
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Unbanned process should be favored
        assert!(processes[0].mix_share < processes[1].mix_share);

        assert_eq!(processes[0].change, ProcessChange::Contracting);
        assert_eq!(processes[1].change, ProcessChange::Expanding);

        // Should be normalized
        approx_eq!(f32, processes[0].mix_share + processes[1].mix_share, 1.0);
    }

    #[test]
    fn test_update_mix_share_priority() {
        let mut processes = gen_processes();
        processes[0].resources.water = 10.;
        processes[1].resources.water = 1.;
        processes[0].resources.land = 1.;
        processes[1].resources.land = 10.;
        processes[0].resources.electricity = 10.;
        processes[1].resources.fuel = 1.;
        processes[0].byproducts.co2 = 1.;
        processes[1].byproducts.co2 = 10.;

        let demand = outputs!(fuel: 100.);
        let resource_weights = resources!();
        let feedstock_weights = feedstocks!();

        let priority = Priority::Land;
        processes[0].mix_share = 0.5;
        processes[1].mix_share = 0.5;
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Less land intensive process should be favored
        assert!(processes[0].mix_share > processes[1].mix_share);

        let priority = Priority::Water;
        processes[0].mix_share = 0.5;
        processes[1].mix_share = 0.5;
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Less water intensive process should be favored
        assert!(processes[1].mix_share > processes[0].mix_share);

        let priority = Priority::Energy;
        processes[0].mix_share = 0.5;
        processes[1].mix_share = 0.5;
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Less water intensive process should be favored
        assert!(processes[1].mix_share > processes[0].mix_share);

        let priority = Priority::Emissions;
        processes[0].mix_share = 0.5;
        processes[1].mix_share = 0.5;
        update_mixes(&mut processes, &demand, &resource_weights, &feedstock_weights, &priority);

        // Less water intensive process should be favored
        assert!(processes[0].mix_share > processes[1].mix_share);
    }
}
