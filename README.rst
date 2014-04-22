Marauder
########

Status
======

|travis-img|


Overview
========

Marauder is turn-based post-apocalyptic hexagonal strategy game
written in Rust_.

|screenshot|


Devblog
=======

Devblog (in Russian) with weekly reports: http://ozkriff.github.io

Image gallery: https://ozkriff.imgur.com/marauder


Building
========

See .travis.yml.

Latest rust-nightly is required.

Download and compile deps (gl-rs, glfw3, glfw-rs, rust-stb-imgae, etc...)::

    ./make_deps.sh

Compile Marauder::

    make

Models, textures, sounds, etc are stored in separate repo.
Marauder expects them in 'data' directory.

Download to 'data' directory::

    git clone --depth=1 https://github.com/ozkriff/marauder-data bin/data

Run Marauder::

    cd bin && ./marauder


How to Play
===========

- Use arrows to move camera and '-'/'+' to zoom;
- Hold RMB to rotate camera;
- Press 'u' to create new unit in current tile;
- Click on friendly unit to select it;
- Click on enemy unit to attack it with selected unit;
- Click on tile to move selected unit there;
- Press 't' to end turn;


Contribute
==========

Feel free to report bugs and patches using GitHub's pull requests
system on `ozkriff/marauder`_.  Any feedback would be much appreciated!


License
=======

Marauder is licensed under the MIT license (see the "LICENSE" file).


.. |travis-img| image:: https://travis-ci.org/ozkriff/marauder.png?branch=master
.. _Rust: https://rust-lang.org
.. |screenshot| image:: http://i.imgur.com/U0iHH5R.gif
.. _`ozkriff/marauder`: https://github.com/ozkriff/marauder
