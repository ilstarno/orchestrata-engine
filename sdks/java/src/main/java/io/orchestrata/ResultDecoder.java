package io.orchestrata;

import java.math.BigInteger;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

final class ResultDecoder {
    private final String input;
    private int index;

    ResultDecoder(String input) {
        this.input = input;
    }

    Optional<Solution> decode(int expectedVariables) {
        take("{");
        take("\"solution\"");
        take(":");
        if (match("null")) {
            finish();
            return Optional.empty();
        }

        take("{");
        List<Long> values = null;
        Score score = null;
        BigInteger evaluated = null;
        do {
            if (values == null && match("\"values\"")) {
                take(":");
                values = values(expectedVariables);
            } else if (score == null && match("\"score\"")) {
                take(":");
                score = score();
            } else if (evaluated == null && match("\"evaluated\"")) {
                take(":");
                evaluated = new BigInteger(integer());
            } else {
                throw invalid();
            }
        } while (match(","));
        take("}");
        finish();
        if (values == null || values.size() != expectedVariables || score == null
            || score.hard() != 0 || score.soft() > 0 || evaluated == null
            || evaluated.signum() <= 0 || evaluated.bitLength() > 64) {
            throw invalid();
        }
        return Optional.of(new Solution(values, score, evaluated));
    }

    private List<Long> values(int expectedVariables) {
        take("[");
        var values = new ArrayList<Long>();
        if (!match("]")) {
            do {
                if (values.size() >= expectedVariables) throw invalid();
                values.add(signedInteger());
            } while (match(","));
            take("]");
        }
        return values;
    }

    private Score score() {
        take("{");
        Long hard = null;
        Long soft = null;
        do {
            if (hard == null && match("\"hard\"")) {
                take(":");
                hard = signedInteger();
            } else if (soft == null && match("\"soft\"")) {
                take(":");
                soft = signedInteger();
            } else {
                throw invalid();
            }
        } while (match(","));
        take("}");
        if (hard == null || soft == null) throw invalid();
        return new Score(hard, soft);
    }

    private long signedInteger() {
        try {
            return Long.parseLong(integer());
        } catch (NumberFormatException failure) {
            throw invalid();
        }
    }

    private String integer() {
        whitespace();
        int start = index;
        if (index < input.length() && input.charAt(index) == '-') index++;
        int digits = index;
        while (index < input.length() && input.charAt(index) >= '0' && input.charAt(index) <= '9') {
            if (index - digits == 20) throw invalid();
            index++;
        }
        if (digits == index || (index - digits > 1 && input.charAt(digits) == '0')) throw invalid();
        return input.substring(start, index);
    }

    private boolean match(String token) {
        whitespace();
        if (!input.startsWith(token, index)) return false;
        index += token.length();
        return true;
    }

    private void take(String token) {
        if (!match(token)) throw invalid();
    }

    private void finish() {
        take("}");
        whitespace();
        if (index != input.length()) throw invalid();
    }

    private void whitespace() {
        while (index < input.length() && " \t\r\n".indexOf(input.charAt(index)) >= 0) index++;
    }

    private IllegalStateException invalid() {
        return new IllegalStateException("Invalid native ABI v1 solve result");
    }
}
