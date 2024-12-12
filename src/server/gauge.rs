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
		<script src="https://code.highcharts.com/highcharts.js"></script>
		<script src="https://code.highcharts.com/highcharts-more.js"></script>
		<script src="https://code.highcharts.com/modules/solid-gauge.js"></script>
		<script src="https://code.highcharts.com/modules/exporting.js"></script>
		<script src="https://code.highcharts.com/modules/export-data.js"></script>
		<script src="https://code.highcharts.com/modules/accessibility.js"></script>
		<script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
		<style>
			.highcharts-figure .chart-container {
			    width: 300px;
			    height: 200px;
			    float: left;
			}

			.highcharts-figure,
			.highcharts-data-table table {
			    width: 600px;
			    margin: 0 auto;
			}

			.highcharts-data-table table {
			    font-family: Verdana, sans-serif;
			    border-collapse: collapse;
			    border: 1px solid #ebebeb;
			    margin: 10px auto;
			    text-align: center;
			    width: 100%;
			    max-width: 500px;
			}

			.highcharts-data-table caption {
			    padding: 1em 0;
			    font-size: 1.2em;
			    color: #555;
			}

			.highcharts-data-table th {
			    font-weight: 600;
			    padding: 0.5em;
			}

			.highcharts-data-table td,
			.highcharts-data-table th,
			.highcharts-data-table caption {
			    padding: 0.5em;
			}

			.highcharts-data-table thead tr,
			.highcharts-data-table tr:nth-child(even) {
			    background: #f8f8f8;
			}

			.highcharts-data-table tr:hover {
			    background: #f1f7ff;
			}

			@media (max-width: 600px) {
			    .highcharts-figure,
			    .highcharts-data-table table {
			        width: 100%;
			    }

			    .highcharts-figure .chart-container {
			        width: 300px;
			        float: none;
			        margin: 0 auto;
			    }
			}

			.highcharts-description {
			    margin: 0.3rem 10px;
			}
		</style>
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
					<div id="container-AB_WgesOut" style="height: 30px; padding: 10px">
						<script type="text/javascript">
							AB_WgesOut = ((0).toFixed(4));
							var str_AB_WgesOut = ('<center><div class="segfontbk">' + AB_WgesOut.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + AB_WgesOut.split(".")[1] + '<\/div>kWh<\/center>');
							document.write(str_AB_WgesOut);
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
 
				// JSON abholen
				setInterval(function () {
					$.ajax({
						type: "GET",
						url: "http://10.15.40.17:3000/now",
 
						success: function(data, status){
						console.log(data);
							//var response = JSON.parse(data);
							var response = data;
							var point,
							newVal,
							inc;
 
							if (chartAB_Pges) {
								point = chartAB_Pges.series[0].points[0];		
								newVal = response.current_net_power;
								point.update(newVal);
							}
 
							if (chartAB_PL1) {
								point = chartAB_PL1.series[0].points[0];
								newVal = response.line_one;
								point.update(newVal);
							}
 
							if (chartAB_PL2) {
								point = chartAB_PL2.series[0].points[0];
								newVal = response.line_two;
								point.update(newVal);
							}
 
							if (chartAB_PL3) {
								point = chartAB_PL3.series[0].points[0];
								newVal = response.line_three;
								point.update(newVal);
							}

							AB_Wges = (response.total_energy_inbound).toFixed(4);
							str_AB_Wges = ('<center><div class="segfontbk">' + AB_Wges.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + AB_Wges.split(".")[1] + '<\/div>kWh<\/center>');
							document.getElementById("container-AB_Wges").innerHTML = str_AB_Wges;

							AB_WgesOut = (response.total_energy_outbound).toFixed(4);
							str_AB_WgesOut = ('<center><div class="segfontbk">' + AB_WgesOut.split(".")[0] + '<\/div><div class="komma">,<\/div><div class="segfontbk">' + AB_WgesOut.split(".")[1] + '<\/div>kWh<\/center>');
							document.getElementById("container-AB_WgesOut").innerHTML = str_AB_WgesOut;
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
