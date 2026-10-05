// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled histogram input to the unmodified official HiLoPeakFinder.
import java.util.*;
import java.nio.file.*;

public class HiLoControlProbe {
    static String ranges(List<?> ranges) throws Exception {
        List<String> result = new ArrayList<>();
        for (Object range : ranges) result.add("{\"min\":" + GridProbe.get(range, "min")
            + ",\"main\":" + GridProbe.get(range, "main") + ",\"max\":" + GridProbe.get(range, "max") + "}");
        return "[" + String.join(",", result) + "]";
    }
    static String observe(String name, int[] values, int origin, int domainStart, int domainStop,
                          int minValue, int derivative, double gain, int quorum) throws Exception {
        Class<?> integerFunction = Class.forName("org.audiveris.omr.math.IntegerFunction");
        Class<?> finderClass = Class.forName("org.audiveris.omr.math.HiLoPeakFinder");
        Object function = integerFunction.getConstructor(int.class, int.class).newInstance(origin, origin + values.length - 1);
        for (int i = 0; i < values.length; i++) GridProbe.call(function, "setValue", origin + i, values[i]);
        Object finder = finderClass.getConstructor(String.class, integerFunction, int.class, int.class)
            .newInstance(name, function, domainStart, domainStop);
        Object threshold = Class.forName("org.audiveris.omr.math.HiLoPeakFinder$Quorum").getConstructor(int.class).newInstance(quorum);
        GridProbe.call(finder, "setQuorum", threshold);
        List<?> peaks = (List<?>) GridProbe.call(finder, "findPeaks", minValue, derivative, gain);
        String returnedOrder = ranges(peaks);
        List<Object> sorted = new ArrayList<>(peaks);
        sorted.sort((a,b) -> { try { return Integer.compare((Integer) GridProbe.get(a,"main"),(Integer) GridProbe.get(b,"main")); }
                              catch(Exception e) { throw new RuntimeException(e); } });
        return "{\"name\":\"" + name + "\",\"function_x_min\":" + origin + ",\"values\":" + Arrays.toString(values)
            + ",\"domain_start\":" + domainStart + ",\"domain_stop\":" + domainStop
            + ",\"min_value\":" + minValue + ",\"min_derivative\":" + derivative + ",\"min_gain_ratio\":" + gain
            + ",\"quorum\":" + quorum + ",\"returned_order\":" + returnedOrder + ",\"peaks\":" + ranges(sorted)
            + ",\"hilos\":" + ranges((List<?>) GridProbe.get(finder, "hilos")) + "}";
    }
    public static void main(String[] args) throws Exception {
        List<String> cases = new ArrayList<>();
        int[][] values = {{0,0,0,0,0}, {0,0,9,0,0}, {0,0,10,0,0}, {0,0,11,0,0},
            {0,3,6,9,6,3,0}, {0,0,10,10,10,0,0}, {10,8,6,4,2,0}, {0,0,2,6,10},
            {0,8,0,8,0,0}, {0,0,12,3,9,0,0}, {0,1,2,4,8,12,8,4,2,1,0},
            {0,0,5,10,5,0,0}, {0,0,10,10,9,8,7,6,5}, {0,2,4,6,8,10}};
        String[] names = {"all-zero", "below-quorum", "at-quorum", "above-quorum", "exact-derivative", "flat-top",
            "left-edge-descent", "terminal-rise", "equal-neighbor-peaks", "unequal-neighbor-peaks", "wide-tail-gain",
            "quorum-equality", "terminal-descent", "monotone-terminal"};
        for (int i = 0; i < values.length; i++) cases.add(observe(names[i], values[i], 0, 0, values[i].length - 1, 2, 3, 0.2, 10));
        cases.add(observe("shifted-domain", values[9], 100, 100, 106, 2, 3, 0.2, 5));
        cases.add(observe("bounded-subdomain", new int[]{30,0,0,10,0,0,30}, 0, 1, 5, 2, 3, 0.2, 5));
        cases.add(observe("zero-derivative", new int[]{0,1,2,1,0}, 0, 0, 4, 1, 0, 0.2, 1));
        // Fixed seed creates varied controls; all exact inputs are persisted with outputs.
        Random random = new Random(5110927L);
        for (int i = 0; i < 96; i++) {
            int[] histogram = new int[4 + random.nextInt(45)];
            for (int j = 0; j < histogram.length; j++) histogram[j] = random.nextInt(25);
            int origin = (i % 3 == 0) ? 100 : 0;
            cases.add(observe("seeded-" + i, histogram, origin, origin, origin + histogram.length - 1,
                1 + random.nextInt(5), 1 + random.nextInt(6), new double[]{0.01,0.1,0.2,0.5}[i % 4], 5 + random.nextInt(16)));
        }
        Files.writeString(Path.of(args[0]), "{\"controlled_input\":true,\"random_seed\":5110927,\"cases\":[" + String.join(",", cases) + "]}\n");
        System.exit(0);
    }
}
