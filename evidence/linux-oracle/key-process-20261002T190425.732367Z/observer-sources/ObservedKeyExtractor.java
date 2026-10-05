// SPDX-License-Identifier: AGPL-3.0-or-later
// Test-only delegating observer: every recognition operation calls original super.
// Compiled against compile-only ABI shims; runtime resolves official JAR classes.
import java.util.*;
import org.audiveris.omr.sheet.Staff;
import org.audiveris.omr.sheet.key.*;
import org.audiveris.omr.sheet.header.StaffHeader;
import org.audiveris.omr.glyph.Shape;
import org.audiveris.omr.sig.inter.KeyAlterInter;

public class ObservedKeyExtractor extends KeyExtractor {
    private final KeyBuilderProcessProbe observer;
    public ObservedKeyExtractor(Staff staff, KeyBuilderProcessProbe observer) { super(staff); this.observer=observer; }
    @Override public void retrieveComponents(StaffHeader.Range range, KeyRoi roi, List<KeyPeak> peaks, Shape shape) {
        super.retrieveComponents(range,roi,peaks,shape);
        observer.componentsObserved(roi,shape);
    }
    @Override public KeyAlterInter extractAlter(KeyRoi roi, List<KeyPeak> peaks, KeySlice slice, Set<Shape> shapes, double minGrade, boolean cropNeighbors) {
        String kind="other";
        for(StackTraceElement element:Thread.currentThread().getStackTrace()) {
            if(element.getMethodName().equals("fillMissingAlters")){kind="fillMissingAlters";break;}
            if(element.getMethodName().equals("extractEmptySlices"))kind="extractEmptySlices";
        }
        String before=observer.extractionBefore(roi,slice,kind,minGrade,cropNeighbors);
        KeyAlterInter result=super.extractAlter(roi,peaks,slice,shapes,minGrade,cropNeighbors);
        observer.extractionAfter(roi,slice,before);
        return result;
    }
}
