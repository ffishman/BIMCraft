@dansity, a note on #449's `view3d::stream`, found while measuring #423 on the Tekla model from #264.

On a 16 GB Mac, when free memory / 4 doesn't cover the scene's estimate, the 3D view keeps only part of the model. The Tekla model has a single level, so what stays is the columns: beams, plates and bolts are left out of the view, with nothing on screen saying so. It already happens on main, without #423.

With #423 the model has every one of its 39,739 bolts, and building the scene takes 25 s against 1.2 s, almost all of it in each copy's triangles and normals, one by one. `docs/3d-acceleration-plan.md` names the fix: the scene holding copies as placements, as the card already does. That would also let this model in whole. Are you on it, or would you like me to try it, the way you prefer? I'd rather agree it with you than touch the scene alongside #460 and #473.
