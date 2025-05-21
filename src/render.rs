//When doing raytracing:
//Keep track of solid angle for each pixel's ray
//If triangle in quadtree is smaller than solid angle, take the average of all the star colors in the triangle and use that as color
//Does this mean maybe the quadtree should be constructed from scratch every launch with the solid angle of the pixels at the set
//resolution in mind? To use as a depth limit for the tree and remove the requirement for the fragment shader to do averages of an
//unbounded number of stars every frame

//Stars themselves have a solid angle (at least a perceptual one based on naked eye fov, that is invariant(doesn't change rel. to screen) to zoom level)