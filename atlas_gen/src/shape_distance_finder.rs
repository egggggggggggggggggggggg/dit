use crate::contour_combiner::SimpleContourCombiner;

struct ShapeDistanceFinder {
    shape: Shape,
    contour_combiner: SimpleContourCombiner<>
}
