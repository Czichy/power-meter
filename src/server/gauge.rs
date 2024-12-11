use std::sync::Arc;

use axum::{http::header, response::Response};
use crossbeam_utils::atomic::AtomicCell;

use crate::meter_reading::MeterReading;

pub async fn handler(latest_reading_cell: Arc<AtomicCell<Option<MeterReading>>>) -> Response {
    let reading = latest_reading_cell.take();

    let status = if reading.is_some() { 200 } else { 204 };

    let body = match reading {
        Some(reading) => 
            r#"
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN""http://www.w3.org/TR/html4/loose.dtd">
<html>
	<head>
		<meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
		<meta name="Author" content="Stefan Weigert">
		<meta name="DESCRIPTION" content="SML Testseite">
		<meta name="PAGE-CONTENT" content="Elektronik">
		<meta name="lang" content="de">
		<meta name="ROBOTS" content="INDEX,FOLLOW">
		<meta name="REVISIT-AFTER" content="60 days">
		<meta name="KeyWords" lang="de" content="SML, Smartmeter, FTDI">
		<title>SML Testseite</title>
		<link href="css/style2.css" rel="stylesheet" type="text/css">
		<script type="text/javascript" src="js/highcharts.js"></script>
		<script type="text/javascript" src="js/highcharts-more.js"></script>
		<script type="text/javascript" src="js/solid-gauge.js"></script>
		<script type="text/javascript" src="js/jquery-3.3.1.min.js"></script>
	</head>
	<body>
		<div id="text_body">
			<div style="width: 790px; height: 470px; margin: 0 auto; background-color: #CCCCCC">
				<div style="width: 489px; background-color: #FFFFFF; float: left">
					<div style="height: 40px">
						<center><h1>allgemeiner Bedarf</h1></center>
					</div>
					<div id="container-AB_Pges" style="height: 200px"></div>
					<div style="height: 175px">
						<div id="container-AB_PL1" style="width: 163px; height: 175px; float: left"></div>
						<div id="container-AB_PL2" style="width: 163px; height: 175px; float: left"></div>
						<div id="container-AB_PL3" style="width: 163px; height: 175px; float: left"></div>
					</div>
					<div id="container-AB_Wges" style="height: 30px; padding: 10px">
						<script type="text/javascript">
							AB_Wges = ((0).toFixed(4));
							var str_AB_Wges = ('<center><div class="segfontbk">' + AB_Wges.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + AB_Wges.split(".")[1] + '<\/div>kWh<\/center>');
							document.write(str_AB_Wges);
						</script>
					</div>
				</div>
 
				<div style="width: 296px; background-color: #FFFFFF; float: right">
					<div style="height: 40px">
						<center><h1>WÃ¤rmepumpe</h1></center>
					</div>
					<div id="container-WP_Pges" style="height: 200px"></div>
					<div style="height: 175px;"></div>
					<div id="container-WP_Wges" style="height: 30px; padding: 10px">
						<script type="text/javascript">
							WP_Wges = ((0).toFixed(4));
							var str_WP_Wges = ('<center><div class="segfontbk">' + WP_Wges.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + WP_Wges.split(".")[1] + '<\/div>kWh<\/center>');
							document.write(str_WP_Wges);
						</script>
					</div>
				</div>
			</div>
 
			<script type="text/javascript">
				// globale Einstellungen der Gauges
				var gaugeOptions = {
					chart: {
						type: 'solidgauge',
						style: {
							fontFamily: 'Dosis, sans-serif'
						}
					},
					title: null,
					pane: {
						center: ['50%', '85%'],
						size: '100%',
						startAngle: -90,
						endAngle: 90,
						background: {
							backgroundColor: (Highcharts.theme && Highcharts.theme.background2) || '#EEE',
							innerRadius: '60%',
							outerRadius: '100%',
							shape: 'arc'
						}
					},
					credits: { enabled: false },
					tooltip: { enabled: false },
					yAxis: {
						stops: [
							[0.1, '#55BF3B'],
							[0.5, '#DDDF0D'],
							[0.9, '#DF5353']
						],
						lineWidth: 0,
						minorTickInterval: null,
						tickAmount: 2,
						labels: { y: 16 }
					},
					plotOptions: {
						solidgauge: {
							dataLabels: {
								y: 15,
								borderWidth: 0,
								useHTML: true
							}
						}
					}
				};
 
				// AB_Pges Gauge
				var chartAB_Pges = Highcharts.chart('container-AB_Pges', Highcharts.merge(gaugeOptions, {
					pane: { size: '150%' },
					yAxis: {
						min: 0,
						max: 6000,
						title: {
							y: -80,			
							style: {
								font: 'bold 16px Dosis, sans-serif',
								color: '#000000',
							},			
							text: 'Gesamtwirkleistung'
						}
					},
					series: [{
						name: 'AB_Pges',
						data: [0],
						dataLabels: {
							format: '<div style="text-align:center"><span style="font-size:30px;' +
								((Highcharts.theme && Highcharts.theme.contrastTextColor) || 'black') + '">{y}<\/span><br>' +
								   '<span style="font-size:22px;color:silver">W<\/span><\/div>'
						},
					}]
				}));
 
				// AB_PL1 Gauge
				var chartAB_PL1 = Highcharts.chart('container-AB_PL1', Highcharts.merge(gaugeOptions, {
					yAxis: {
						min: 0,
						max: 2000,
						title: {
							y: -50,
							style: {
								font: 'bold 16px Dosis, sans-serif',
								color: '#000000',
							},
							text: 'Wirkleistung L1'
						}
					},
					series: [{
						name: 'AB_PL1',
						data: [0],
						dataLabels: {
							format: '<div style="text-align:center"><span style="font-size:20px;' +
								((Highcharts.theme && Highcharts.theme.contrastTextColor) || 'black') + '">{y}<\/span><br>' +
								   '<span style="font-size:16px;color:silver">W<\/span><\/div>'
						},
						tooltip: {
							valueSuffix: ' W'
						}
					}]
				}));
 
				// AB_PL2 gauge
				var chartAB_PL2 = Highcharts.chart('container-AB_PL2', Highcharts.merge(gaugeOptions, {
					yAxis: {
						min: 0,
						max: 2000,
						title: {
							y: -50,
							style: {
								font: 'bold 16px Dosis, sans-serif',
								color: '#000000',
							},
							text: 'Wirkleistung L2'
						}
					},
					series: [{
						name: 'AB_PL2',
						data: [0],
						dataLabels: {
							format: '<div style="text-align:center"><span style="font-size:20px;' +
								((Highcharts.theme && Highcharts.theme.contrastTextColor) || 'black') + '">{y}<\/span><br>' +
								   '<span style="font-size:16px;color:silver">W<\/span><\/div>'
						},
					}]
				}));
 
				// AB_PL3 gauge
				var chartAB_PL3 = Highcharts.chart('container-AB_PL3', Highcharts.merge(gaugeOptions, {
					yAxis: {
						min: 0,
						max: 2000,
						title: {
							y: -50,
							style: {
								font: 'bold 16px Dosis, sans-serif',
								color: '#000000',
							},
							text: 'Wirkleistung L3'
						}
					},
					series: [{
						name: 'AB_PL3',
						data: [0],
						dataLabels: {
							format: '<div style="text-align:center"><span style="font-size:20px;' +
								((Highcharts.theme && Highcharts.theme.contrastTextColor) || 'black') + '">{y}<\/span><br>' +
								   '<span style="font-size:16px;color:silver">W<\/span><\/div>'
						},
					}]
				}));
 
				// WP_Pges gauge
				var chartWP_Pges = Highcharts.chart('container-WP_Pges', Highcharts.merge(gaugeOptions, {
					pane: { size: '150%' },
 
					yAxis: {
						min: 0,
						max: 2000,
						title: {
							y: -80,			
							style: {
								font: 'bold 16px Dosis, sans-serif',
								color: '#000000',
							},		
							text: 'Gesamtwirkleistung'
						}
					},
					series: [{
						name: 'WP_Pges',
						data: [0],
						dataLabels: {
							format: '<div style="text-align:center"><span style="font-size:30px;color:' +
								((Highcharts.theme && Highcharts.theme.contrastTextColor) || 'black') + '">{point.y:,.1f}<\/span><br>' +
								   '<span style="font-size:22px;color:silver">W<\/span><\/div>'
						},
					}]
				}));
 
				// JSON abholen
				setInterval(function () {
					$.ajax({
						type: "GET",
						url: http://10.15.40.17:3000/now",
 
						success: function(data, status){
							var response = JSON.parse(data);
							var point,
							newVal,
							inc;
 
							if (chartAB_Pges) {
								point = chartAB_Pges.series[0].points[0];		
								newVal = response.AB_Pges;
								point.update(newVal);
							}
 
							if (chartAB_PL1) {
								point = chartAB_PL1.series[0].points[0];
								newVal = response.AB_PL1;
								point.update(newVal);
							}
 
							if (chartAB_PL2) {
								point = chartAB_PL2.series[0].points[0];
								newVal = response.AB_PL2;
								point.update(newVal);
							}
 
							if (chartAB_PL3) {
								point = chartAB_PL3.series[0].points[0];
								newVal = response.AB_PL3;
								point.update(newVal);
							}
 
							AB_Wges = (response.AB_Wges).toFixed(4);
							str_AB_Wges = ('<center><div class="segfontbk">' + AB_Wges.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + AB_Wges.split(".")[1] + '<\/div>kWh<\/center>');
							document.getElementById("container-AB_Wges").innerHTML = str_AB_Wges;
						}
					});
					$.ajax({
						type: "GET",
						url: http://10.15.40.17:3000/now",
 
						success: function(data, status){
							var response = JSON.parse(data);
							var point,
							newVal,
							inc;
 
							if (chartWP_Pges) {
								point = chartWP_Pges.series[0].points[0];		
								newVal = response.WP_Pges;
								point.update(newVal);
							}
 
							WP_Wges = (response.WP_Wges).toFixed(4);
							str_WP_Wges = ('<center><div class="segfontbk">' + WP_Wges.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + WP_Wges.split(".")[1] + '<\/div>kWh<\/center>');
							document.getElementById("container-WP_Wges").innerHTML = str_WP_Wges;
						}
					});
				}, 2000);						
			</script>			
		</div>
	</body>
</html>
"#.to_string(),

        // format!("{reading}"),
        None => "".to_string(),
    };

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html")
        .body(body.into())
        .unwrap()
}