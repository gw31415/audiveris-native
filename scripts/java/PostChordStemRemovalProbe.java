// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class PostChordStemRemovalProbe extends FoundationOverlapProbe {
    static Object observe(Object sig,Object h,Object s,Object chord)throws Exception {
        List<Object> members=new ArrayList<>();for(Object n:(Collection<?>)call(chord,"getMembers"))members.add(n==h?"head":"other");
        Object stem=call(chord,"getStem");return obj("chord_kind",chord.getClass().getSimpleName(),
            "chord_removed",call(chord,"isRemoved"),"head_removed",call(h,"isRemoved"),
            "stem_removed",call(s,"isRemoved"),"stem",stem==null?null:"stem",
            "members",members,"bounds",rect(call(chord,"getBounds")),"grade",call(chord,"getGrade"),
            "head_location",point(call(chord,"getHeadLocation")),"tail_location",point(call(chord,"getTailLocation")),
            "live_vertices",((Collection<?>)call(sig,"vertexSet")).size(),"live_relations",((Collection<?>)call(sig,"edgeSet")).size());
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
        for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
        Object h=head(sys,physical(300,250,12,8,"VERTICAL",null));call(h,"setGrade",.6);call(h,"setContextualGrade",.75);
        Object s=make("sig.inter.StemInter",physical(312,200,2,58,"VERTICAL",null),.8);
        Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));
        for(Object n:List.of(h,s,chord))call(sig,"addVertex",n);call(chord,"addMember",h);
        call(sig,"addEdge",chord,s,make("sig.relation.ChordStemRelation"));
        Object before=observe(sig,h,s,chord);call(s,"remove");Object after=observe(sig,h,s,chord);call(s,"remove");Object again=observe(sig,h,s,chord);
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "scope","original-only next-unit post-chord Stem vertex removal; declared actual ChordStem callback, no HeadStem relations/cache hydration or raster/export parity",
            "head_bounds",new int[]{300,250,12,8},"stem_bounds",new int[]{312,200,2,58},
            "before",before,"after",after,"repeat_is_idempotent",new ObjectMapper().valueToTree(after).equals(new ObjectMapper().valueToTree(again))));System.exit(0);
    }
}
