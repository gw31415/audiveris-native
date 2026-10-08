# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import tempfile,unittest,sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from compare_image_score_events import compare,truth_events,xml_events
TRUTH='''<museScore version="3.01"><Score><Staff id="1"><Measure><voice><TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig><Chord><durationType>quarter</durationType><Note><pitch>60</pitch></Note><Note><pitch>64</pitch></Note></Chord><Chord><durationType>eighth</durationType><appoggiatura/><Note><pitch>67</pitch></Note></Chord><Rest><durationType>quarter</durationType></Rest></voice></Measure></Staff><Staff id="2"><Measure><voice><TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig><Chord><durationType>half</durationType><Note><pitch>50</pitch></Note></Chord></voice></Measure></Staff></Score></museScore>'''
XML='''<score-partwise><part id="P1"><measure><attributes><divisions>4</divisions><staves>2</staves><time><beats>4</beats><beat-type>4</beat-type></time></attributes><note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><voice>1</voice><staff>1</staff></note><note><chord/><pitch><step>E</step><octave>4</octave></pitch><duration>4</duration><voice>1</voice><staff>1</staff></note><note><grace/><pitch><step>G</step><octave>4</octave></pitch><voice>1</voice><staff>1</staff></note><note><rest/><duration>4</duration><voice>1</voice><staff>1</staff></note><backup><duration>8</duration></backup><note><pitch><step>D</step><octave>3</octave></pitch><duration>8</duration><voice>5</voice><staff>2</staff></note></measure></part></score-partwise>'''
class MusicalEventComparison(unittest.TestCase):
    def test_chord_grace_rest_backup_and_staff_timing_agree_without_claiming_voice_notation_acceptance(self):
        with tempfile.TemporaryDirectory() as d:
            truth=Path(d)/'truth.mscx';xml=Path(d)/'read.musicxml';truth.write_text(TRUTH);xml.write_text(XML)
            expected=[(1,1,'0',60,'1/4',False),(1,1,'0',64,'1/4',False),(1,1,'1/4',67,'0',True),(1,1,'1/4',None,'1/4',False),(2,1,'0',50,'1/2',False)]
            self.assertEqual(truth_events(truth)[0],expected);self.assertEqual(xml_events(xml)[0],expected)
            result=compare(truth,xml);self.assertEqual(result['matched_events'],5);self.assertEqual(result['F1'],1);self.assertFalse(result['quality_accepted'])
    def test_pitch_error_creates_one_missing_and_one_extra_event(self):
        with tempfile.TemporaryDirectory() as d:
            truth=Path(d)/'truth.mscx';xml=Path(d)/'read.musicxml';truth.write_text(TRUTH);xml.write_text(XML.replace('<step>D</step>','<step>E</step>'))
            result=compare(truth,xml);self.assertEqual(result['matched_events'],4);self.assertEqual(result['precision'],.8);self.assertEqual(len(result['missing_events']),1);self.assertEqual(len(result['extra_events']),1);self.assertFalse(result['quality_accepted'])
if __name__=='__main__':unittest.main()
