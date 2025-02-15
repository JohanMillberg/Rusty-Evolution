use rand::seq::{index, IndexedRandom, SliceRandom};
use rand::{Rng, RngCore};
use std::ops::Index;

#[derive(Clone, Debug)]
pub struct Chromosome {
    genes: Vec<f32>,
}

impl Chromosome {
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.genes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.genes.iter_mut()
    }
}

pub trait Individual {
    fn fitness(&self) -> f32;
    fn chromosome(&self) -> &Chromosome;
}

pub trait CrossoverMethod {
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome,
    ) -> Chromosome;
}

#[derive(Clone, Debug)]
pub struct UniformCrossover;

impl CrossoverMethod for UniformCrossover {
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome,
    ) -> Chromosome {
        assert_eq!(parent_a.len(), parent_b.len());

        parent_a
            .iter()
            .zip(parent_b.iter())
            .map(|(&a, &b)| if rng.random_bool(0.5) { a } else { b })
            .collect()
    }
}

pub trait SelectionMethod {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual;
}

impl Index<usize> for Chromosome {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.genes[index]
    }
}

impl FromIterator<f32> for Chromosome {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self {
            genes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Chromosome {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

pub struct RouletteWheelSelection;

impl SelectionMethod for RouletteWheelSelection {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual,
    {
        population
            .choose_weighted(rng, |individual| individual.fitness())
            .expect("got an empty population")
    }
}

pub struct GeneticAlgorithm<S> {
    selection_method: S,
    crossover_method: Box<dyn CrossoverMethod>,
}

impl<S> GeneticAlgorithm<S>
where
    S: SelectionMethod,
{
    pub fn new(selection_method: S, crossover_method: impl CrossoverMethod + 'static) -> Self {
        Self {
            selection_method,
            crossover_method: Box::new(crossover_method),
        }
    }

    pub fn evolve<I>(&self, rng: &mut dyn RngCore, population: &[I]) -> Vec<I>
    where
        I: Individual,
    {
        assert!(!population.is_empty());
        (0..population.len())
            .map(|_| {
                let parent_a = self.selection_method.select(rng, population).chromosome();
                let parent_b = self.selection_method.select(rng, population).chromosome();
                let mut child = self.crossover_method.crossover(rng, parent_a, parent_b);
                todo!("Code mutation")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    #[derive(Clone, Debug)]
    struct TestIndividual {
        fitness: f32,
    }

    impl TestIndividual {
        fn new(fitness: f32) -> Self {
            Self { fitness }
        }
    }

    impl Individual for TestIndividual {
        fn fitness(&self) -> f32 {
            self.fitness
        }
        fn chromosome(&self) -> &Chromosome {
            panic!("not implemented for test individuals")
        }
    }

    #[test]
    fn uniform_crossover() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());
        let parent_a: Chromosome = (1..=100).map(|n| n as f32).collect();
        let parent_b: Chromosome = (1..=100).map(|n| -n as f32).collect();
        let child = UniformCrossover.crossover(&mut rng, &parent_a, &parent_b);

        let diff_a = child.iter().zip(parent_a).filter(|(c, p)| *c != p).count();
        let diff_b = child.iter().zip(parent_b).filter(|(c, p)| *c != p).count();

        assert_eq!(diff_a, 49);
        assert_eq!(diff_b, 51);
    }

    #[test]
    fn roulette_wheel_selection() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());
        let population = vec![
            TestIndividual::new(2.0),
            TestIndividual::new(1.0),
            TestIndividual::new(4.0),
            TestIndividual::new(3.0),
        ];

        let mut actual_histogram = BTreeMap::new();

        for _ in 0..1000 {
            let fitness = RouletteWheelSelection
                .select(&mut rng, &population)
                .fitness() as i32;
            *actual_histogram.entry(fitness).or_insert(0) += 1;
        }

        let expected_histogram = BTreeMap::from_iter([(1, 98), (2, 202), (3, 278), (4, 422)]);

        assert_eq!(actual_histogram, expected_histogram)
    }
}
