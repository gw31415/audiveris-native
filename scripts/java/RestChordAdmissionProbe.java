// SPDX-License-Identifier: AGPL-3.0-or-later
// Original RestInter.createValid proximity to a real stem-bound HeadChord.
import java.nio.file.Path;
import java.lang.reflect.Field;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RestChordAdmissionProbe extends FoundationOverlapProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
        for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
        Object head=head(sys,physical(300,250,12,8,"VERTICAL",null)),stem=make("sig.inter.StemInter",physical(312,200,2,58,"VERTICAL",null),.8),chord=make("sig.inter.HeadChordInter",.8);
        for(Object n:List.of(head,stem,chord))call(sig,"addVertex",n);call(chord,"setStaff",call(sys,"getFirstStaff"));call(chord,"addMember",head);call(chord,"setStem",stem);
        Field f=c("sig.inter.RestInter").getDeclaredField("constants");f.setAccessible(true);Object fraction=get(f.get(null),"minInterChordDx");Object minDx=call(call(sheet,"getScale"),"toPixels",fraction);
        int[][] bounds={{300,248,11,8},{300,210,11,8},{295,210,11,8},{296,210,11,8},{300,192,11,8}};
        String[] names={"near-head","near-stem-tail-outside-head-box","exact-minimum-dx","below-minimum-dx","tail-boundary-only"};
        List<Object> cases=new ArrayList<>();
        for(int i=0;i<names.length;i++) {
            int[] b=bounds[i];Object glyph=physical(b[0],b[1],b[2],b[3],"VERTICAL",null);
            Object rest=c("sig.inter.RestInter").getMethod("createValid",c("glyph.Glyph"),c("glyph.Shape"),double.class,c("sheet.SystemInfo"),List.class).invoke(null,glyph,c("glyph.Shape").getField("QUARTER_REST").get(null),.8,sys,List.of(chord));
            cases.add(obj("name",names[i],"rest_bounds",b,"accepted",rest!=null,"rest_staff_id",rest==null?null:call(call(rest,"getStaff"),"getId")));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","five original RestInter.createValid near-chord admission conditions on saved ordinary staff/measure and actual HeadChord/Containment/ChordStem; not classifier/raster/MusicXML recognition","interline",call(call(sheet,"getScale"),"getInterline"),"minimum_dx",minDx,"head_bounds",rect(call(head,"getBounds")),"stem_bounds",rect(call(stem,"getBounds")),"chord_bounds",rect(call(chord,"getBounds")),"cases",cases));System.exit(0);
    }
}
