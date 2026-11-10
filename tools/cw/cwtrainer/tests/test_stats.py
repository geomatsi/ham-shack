import json

from cwtrainer.grading import grade
from cwtrainer.scenarios.base import Result
from cwtrainer.stats import Stats


def res(expected, sent):
    return Result.from_grade(grade(expected, sent))


def test_record_and_weak(tmp_path):
    st = Stats(tmp_path / "s.json")
    st.start_session("echo", 20)
    for _ in range(5):
        st.record(res("TNX", "TNK"))
    st.record(res("NN", "N"))
    assert st.chars["X"].attempts == 5 and st.chars["X"].errors == 5
    assert st.chars["N"].attempts == 7 and st.chars["N"].errors == 1
    assert st.weak_characters(3) == ["X", "N"]
    assert st.weak_characters(3, min_attempts=10) == []
    s = st.end_session()
    assert s.prompts == 6 and s.perfect == 0


def test_persist(tmp_path):
    p = tmp_path / "sub" / "s.json"
    st = Stats(p)
    st.start_session("echo", 22)
    st.record(res("CQ", "CQ"))
    st.save()
    assert len(json.loads(p.read_text())["sessions"]) == 1  # in-progress session saved
    st.end_session()
    st.save()
    st2 = Stats(p)
    assert st2.chars["C"].attempts == 1
    assert st2.sessions[0]["perfect"] == 1 and st2.sessions[0]["wpm"] == 22
    assert len(st2.sessions) == 1


def test_no_path():
    st = Stats(None)
    st.record(res("E", "E"))
    st.save()
