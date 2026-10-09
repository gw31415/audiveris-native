// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

class ClefAbsoluteControls {
    private static Class<?> clefClass, shapeClass;
    private static Constructor<?> clefConstructor, headConstructor;
    private static Method setPitch, integerPitch, absolute, step, octave;

    static String observe(String shape, Double clefPitch, double headPitch, String group) throws Exception {
        Object clef = null;
        if (shape != null) {
            clef = clefConstructor.newInstance(shapeClass.getField(shape).get(null), 1.0);
            setPitch.invoke(clef, clefPitch);
        }
        Object head = headConstructor.newInstance(new Rectangle(0, 0, 1, 1),
            shapeClass.getField("NOTEHEAD_BLACK").get(null), 1.0, null, headPitch);
        int rounded = (Integer) integerPitch.invoke(head);
        // Each answer comes directly from its original API. Musical step/octave
        // do not reconstruct the absolute-pitch result.
        Object actualAbsolute = absolute.invoke(null, clef, rounded);
        Object actualStep = step.invoke(null, clef, rounded);
        Object actualOctave = octave.invoke(null, clef, headPitch);
        return "{\"group\":\"" + group + "\",\"shape\":"
            + (shape == null ? "null" : "\"" + shape + "\"")
            + ",\"clef_pitch\":" + clefPitch + ",\"head_pitch\":" + headPitch
            + ",\"head_integer_pitch\":" + rounded + ",\"absolute_pitch\":" + actualAbsolute
            + ",\"step\":\"" + actualStep + "\",\"octave\":" + actualOctave + "}";
    }

    public static void main(String[] args) throws Exception {
        clefClass = Class.forName("org.audiveris.omr.sig.inter.ClefInter");
        shapeClass = Class.forName("org.audiveris.omr.glyph.Shape");
        Class<?> headClass = Class.forName("org.audiveris.omr.sig.inter.HeadInter");
        Class<?> staffClass = Class.forName("org.audiveris.omr.sheet.Staff");
        clefConstructor = clefClass.getConstructor(shapeClass, Double.class);
        headConstructor = headClass.getConstructor(Rectangle.class, shapeClass, Double.class, staffClass, Double.class);
        setPitch = clefClass.getMethod("setPitch", Double.class);
        integerPitch = headClass.getMethod("getIntegerPitch");
        absolute = clefClass.getMethod("absolutePitchOf", clefClass, int.class);
        step = clefClass.getMethod("noteStepOf", clefClass, int.class);
        octave = clefClass.getMethod("octaveOf", clefClass, double.class);
        List<String> rows = new ArrayList<>();
        double[] heads = {-10, -5.5, -4.5, -2, -1.5, -.5, 0, .5, 1.5, 2, 4, 10};
        for (String shape : new String[] {null, "G_CLEF", "G_CLEF_SMALL", "G_CLEF_8VA", "G_CLEF_8VB",
                "F_CLEF", "F_CLEF_SMALL", "F_CLEF_8VA", "F_CLEF_8VB", "C_CLEF", "PERCUSSION_CLEF"}) {
            Double[] clefs = shape == null ? new Double[] {null}
                : shape.equals("C_CLEF") ? new Double[] {-4., -2., -1.5, -.5, 0., .5, 1.5, 2., 4.}
                : new Double[] {shape.startsWith("G") ? 2. : shape.startsWith("F") ? -2. : 0.};
            for (Double clef : clefs)
                for (double head : heads) rows.add(observe(shape, clef, head, "shape-and-half-even"));
        }
        for (double boundary : new double[] {-1.5, -.5, .5, 1.5}) {
            for (double value : new double[] {Math.nextDown(boundary), Math.nextUp(boundary)}) {
                rows.add(observe("C_CLEF", value, 0, "clef-pitch-boundary-ulp"));
                for (String shape : new String[] {null, "G_CLEF", "F_CLEF", "C_CLEF"}) {
                    Double clef = shape == null ? null : shape.equals("C_CLEF") ? 4. : shape.equals("F_CLEF") ? -2. : 2.;
                    rows.add(observe(shape, clef, value, "head-pitch-boundary-ulp"));
                }
            }
        }
        Files.writeString(Path.of(args[0]),
            "{\"scope\":\"Original declared scalar HeadInter.getIntegerPitch then direct static ClefInter.absolutePitchOf/noteStepOf/octaveOf, with null or original ghost ClefInter and declared pitch; no image, Staff, Chord, Measure, source graph or native inputs\",\"absolute_pitch_derived_from_step_octave\":false,\"native_outputs_used\":false,\"cases\":["
                + String.join(",", rows) + "]}\n");
    }
}
