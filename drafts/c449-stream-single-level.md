@dansity, a note on #449's `view3d::stream`, found while measuring #423 on the Tekla model from #264.

On a 16 GB Mac, when free memory / 4 doesn't cover the scene's estimate, the 3D view keeps only part of the model. The Tekla model has a single level, so what stays is the columns: beams, plates and bolts are left out of the view, with nothing on screen saying so. It already happens on main, without #423.

Two thoughts, if they help: choosing what to keep by something other than level when there's only one, and a line in the view saying that part of the model isn't drawn. Happy to test a change on this model.
