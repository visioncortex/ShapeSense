# Shape Completion by Curve Stitching

The ShapeCompletion project aims at recovering (completing) the **structure** of a shape after a portion of it is erased. In other words, given a shape with arbitrarily "discontinued parts", assuming that we have **no prior knowledge** about the shape, how can we **reasonably connect the endpoints** at the discontinuities?

![Hole showcase](images/hole_showcase.png)
A shape with a hole. There are 6 endpoints in this case.

Demo images will be used throughout this documentation. Black pixels denote the background (which is completely ignored). Red pixels denote the rasterized *shape*. White pixels denote the *hole*. The (imaginary) intersection of the shape and the hole is the aforementioned "discontinued parts", which is what we try to recover.

![Recovered shape showcase](images/recovered_shape.png)
An example of completed shape.

Blue pixels will be used to denote the outline of the recovered parts of the shape.

The whole process of shape completion involves intrapolating the missing outline and then filling the pixels in the hole with appropriate colors.

## Simple Shape Completion

Let's begin the experiment with simple shapes, like an ellipse.

![Ellipse ground truth](images/simple/ellipse_groundtruth.png)
![Ellipse with hole](images/simple/ellipse_with_hole.png)
The first image is the ground truth (for reference only). The second image is the input to the ShapeCompletion pipeline.

### Path Preprocessing

The first stage of the pipeline is to obtain and process the paths (curves) representing the existing outline of the shape.

[Vision Cortex's library](https://github.com/visioncortex/visioncortex) provides the necessary utilities to extract raw paths from an image.

![Path segments after preprocessing](images/simple/ellipse_preprocessed_process.png)
Yellow pixels denote the identified outline of the shape after simplification.

### Tail Tangent Approximation

The first step in the pipeline is to extract the two curves from the two endpoints; smoothing is performed to better approximate the tangents near the endpoints (*tails* of the whole curve). After this step, we will obtain two tangents (2-D direction vectors), one at each tail. We will call these tangents *tail tangents*.

![Tail tangent approximation](images/simple/tail_tangent_approx.png)
After discrete rasterization, even a theoretically smooth curve will contain sharp corners. The naive approach is to simply take A as the tail tangent, but better (more practical/useful) approximations may be obtained by taking more subsequent segments into account (e.g. B,C,D).

A number of factors determine the accuracy and robustness of tail tangent approximation. Our implementation takes into account how many points to consider from the tails, how long should the segments being considered accumulate to, and how the weights for each segment should differ towards the tails.

### Intrapolation

If we considered the existing outline of the shape as separate curves at each endpoint, we would be doing *<b>inter</b>polation* **between** curves. However, in this project, we are focusing on curves that form an outline of a shape, so we argue that we are doing *<b>intra</b>polation* **within** curves.