#!/bin/bash
echo "Crashy service: I am alive! (PID: $$)"
sleep 3
echo "Crashy service: I am dying!"
exit 1