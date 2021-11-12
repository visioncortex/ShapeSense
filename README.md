# Shape Completion by Curve Stitching

The ShapeCompletion project aims at recovering (completing) the **structure** of a shape after a portion of it is erased. In other words, given a shape with arbitrarily "discontinued parts", assuming that we have **no prior knowledge** about the shape, how can we **reasonably connect the endpoints** at the discontinuities?

![Hole showcase](images/hole_showcase.png)
A shape with a hole. There are 6 endpoints in this case.

Demo images will be used throughout this documentation. Black pixels denote the background (which is completely ignored). Red pixels denote the rasterized *shape*. White pixels denote the *hole*. The (imaginary) intersection of the shape and the hole is the aforementioned "discontinued parts", which is what we try to recover.

![Recovered shape showcase](images/recovered_shape.png)
An example of completed shape.

Blue pixels will be used to denote the outline of the recovered parts of the shape.

The whole process of shape completion involves intrapolating the missing outline and then filling the pixels in the hole with appropriate colors.
