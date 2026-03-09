import re
from PyPDF2 import PdfReader

PDF_FILE = "data/test-grfc-v1.pdf"


def extract_text(reader, start, end):
    text = ""
    for i in range(start-1, end):
        page = reader.pages[i]
        t = page.extract_text()
        if t:
            text += t + "\n"
    return text


def parse_answers(answer_text):
    """
    Parse answer table from pages 105–106.
    Format example:
    1-d 2-a 3-c ...
    """

    answers = {}

    pairs = re.findall(r'\[(\d+)\]\s*([abcd])', answer_text, re.I)

    for q, a in pairs:
        answers[int(q)] = a.lower()

    return answers


def parse_questions(text):

    questions = []

    blocks = re.split(r'Вопрос\s*№\s*(\d+)', text)

    for i in range(1, len(blocks), 2):

        qid = int(blocks[i])
        body = blocks[i+1].strip()

        lines = body.split("\n")

        text_lines = []
        answers = {}

        current = None

        for line in lines:

            line = line.strip()

            m = re.match(r'^([abcd])\)\s*(.*)', line, re.I)

            if m:
                current = m.group(1).lower()
                answers[current] = m.group(2)
                continue

            if current:
                answers[current] += " " + line
            else:
                text_lines.append(line)

        qtext = " ".join(text_lines).strip()

        questions.append({
            "id": qid,
            "text": qtext,
            "answers": answers
        })

    return questions


def main():

    reader = PdfReader(PDF_FILE)

    questions_text = extract_text(reader, 2, 104)
    answers_text = extract_text(reader, 105, 106)

    answers = parse_answers(answers_text)

    questions = parse_questions(questions_text)

    for q in questions:
        q["correct"] = answers.get(q["id"], "?")

    print("const questions = [")

    for q in questions:

        print("{")
        print(f"id:{q['id']},")

        text = q["text"].replace('"', '\\"')

        print(f'text:"{text}",')

        print("answers:{")

        for k in ["a", "b", "c", "d"]:
            v = q["answers"].get(k, "").replace('"', '\\"')
            print(f'{k}:"{v}",')

        print("},")

        print(f'correct:"{q["correct"]}"')
        print("},")

    print("];")


if __name__ == "__main__":
    main()
