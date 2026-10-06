// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.geom.Point2D;
import com.fasterxml.jackson.databind.ObjectMapper;

public class CandidateReferenceProbe extends MappingRetryProbe {
    static Object compound(Object measure,int id,List<Object> chords) throws Exception {
        Object narrow=make("sheet.rhythm.Slot$MeasureSlot",id,measure,chords);
        return make("sheet.rhythm.Slot$CompoundSlot",id,measure,List.of(narrow));
    }
    static Object inspectReference(Object measure, boolean alias) throws Exception {
        call(measure,"resetRhythm");
        List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
        List<Object> objects=new ArrayList<>();
        objects.add(compound(measure,1,List.of(chords.get(0))));
        objects.add(compound(measure,2,List.of(chords.get(1))));
        List<Object> constructors=new ArrayList<>(List.of(obj("id",1,"chord_ids",List.of(0)),obj("id",2,"chord_ids",List.of(1))));
        List<Integer> ordered=new ArrayList<>(List.of(0,1));
        if(alias) {
            objects.add(compound(measure,3,List.of(chords.get(0),chords.get(2))));
            constructors.add(obj("id",3,"chord_ids",List.of(0,2)));
            ordered.add(2);
        } else {
            ((List)call(objects.get(0),"getChords")).add(chords.get(1));
            ordered.remove(Integer.valueOf(1));
        }
        IdentityHashMap<Object,Integer> keys=new IdentityHashMap<>();
        for(int i=0;i<objects.size();i++)keys.put(objects.get(i),i);
        List<Object> references=new ArrayList<>();
        for(int i=0;i<(alias?3:2);i++) {
            Object slot=call(chords.get(i),"getSlot");
            references.add(obj("chord",i,"object_key",keys.get(slot),"x_offset",call(slot,"getXOffset"),
                    "compare_to_first",call(slot,"compareTo",objects.get(0))));
        }
        Object staff=call(call(measure,"getPart"),"getFirstStaff");
        Point2D left=(Point2D)call(measure,"getSidePoint",c("util.HorizontalSide").getField("LEFT").get(null),staff);
        return obj("name",alias?"later-constructor-rebinds-shared-chord":"membership-add-and-removal-retain-reference-and-cache",
                "input",obj("chord_bounds",chords.stream().map(ch->{try{return rect(call(ch,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),
                        "left",left.getX(),"constructors",constructors,"append_to_first",alias?null:1,"remove_ordered",alias?null:1),
                "output",obj("ordered_keys",ordered,"first_x_offset",call(objects.get(0),"getXOffset"),"references",references));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
        Object sys=((List<?>)call(sheet,"getSystems")).get(0);
        Object measure=call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope","two declared original Slot constructor/reference/cache controls on saved actual chords; list mutation is not full mergeWithPreviousSlot/image polyphony",
                        "controls",List.of(inspectReference(measure,false),inspectReference(measure,true))));
        System.exit(0);
    }
}
