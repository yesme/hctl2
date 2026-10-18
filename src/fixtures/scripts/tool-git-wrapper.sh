#!/bin/sh
if [ "$HCTL2_TEST_MODE" = checkout-after-prepare ] && [ "$3" = update-ref ] && [ "$7" = --create-reflog ]; then
    "$HCTL2_TEST_GIT" "$@" || exit $?
    "$HCTL2_TEST_GIT" -C "$2" worktree add "$HCTL2_TEST_WORKTREE" main || exit $?
    exit 0
fi
if [ "$3" = update-ref ] && [ "$7" = refs/heads/main ]; then
    case "$HCTL2_TEST_MODE" in
        before)
            : > "$HCTL2_TEST_READY"
            while [ ! -f "$HCTL2_TEST_RELEASE" ]; do sleep 0.01; done
            ;;
        after)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            : > "$HCTL2_TEST_READY"
            while [ ! -f "$HCTL2_TEST_RELEASE" ]; do sleep 0.01; done
            exit 0
            ;;
        read-failure)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            : > "$HCTL2_TEST_READY"
            exit 0
            ;;
        advanced-after|failed-and-advanced-after|ancestry-failure)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            "$HCTL2_TEST_GIT" -C "$2" update-ref refs/heads/main "$HCTL2_TEST_OTHER" "$8" || exit $?
            if [ "$HCTL2_TEST_MODE" = failed-and-advanced-after ]; then exit 73; fi
            exit 0
            ;;
        rejected)
            echo 'injected target update rejection' >&2
            exit 73
            ;;
        failed-after)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            exit 73
            ;;
    esac
fi
if [ "$HCTL2_TEST_MODE" = ancestry-failure ] && [ "$3" = merge-base ] && [ "$5" = "$HCTL2_TEST_RESULT" ] && [ "$6" = "$HCTL2_TEST_OTHER" ]; then
    echo 'injected unreadable ancestry' >&2
    exit 73
fi
if [ "$HCTL2_TEST_MODE" = read-failure ] && [ "$3" = for-each-ref ] && [ "$5" = refs/heads/main ] && [ -f "$HCTL2_TEST_READY" ]; then
    echo 'injected unreadable target' >&2
    exit 73
fi
exec "$HCTL2_TEST_GIT" "$@"
