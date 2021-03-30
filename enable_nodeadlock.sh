#!/bin/sh

enable_no_deadlock() {
    for f in $(find src -name *.rs)
    do
	sed -ibak -E "s/std::sync::RwLock(;|,|$)/no_deadlocks::RwLock\1/g" $f
    done
}

disable_no_deadlock() {
    for f in $(find src -name *.rs)
    do
	sed -ibak -E "s/no_deadlocks::RwLock(;|,|$)/std::sync::RwLock\1/g" $f
    done
}

