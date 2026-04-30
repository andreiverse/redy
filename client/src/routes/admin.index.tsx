import { createFileRoute } from '@tanstack/react-router'
import { $api } from '#/lib/api'
import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '#/components/ui/card'
import { type ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent, ChartLegend, ChartLegendContent } from '#/components/ui/chart'
import { Line, LineChart, CartesianGrid, XAxis, YAxis } from 'recharts'
import { Activity, PlayCircle } from 'lucide-react'

export const Route = createFileRoute('/admin/')({
  component: AdminIndex,
})

type HistoryPoint = {
  time: string
  [key: string]: number | string
}

function AdminIndex() {
  const [history, setHistory] = useState<HistoryPoint[]>([])
  
  const { data: stats, isLoading, dataUpdatedAt } = $api.useQuery('get', '/workers/stats', {}, {
    refetchInterval: 5000,
  })

  useEffect(() => {
    if (stats) {
      const now = new Date().toLocaleTimeString()
      const newPoint: HistoryPoint = { time: now }
      
      stats.forEach(stream => {
        // Track total queued in stream
        newPoint[`${stream.stream_name}_total`] = stream.messages
        
        stream.consumers.forEach(consumer => {
          newPoint[`${stream.stream_name}_${consumer.name}_pending`] = consumer.pending
        })
      })

      setHistory(prev => {
        // Only add if it's a new timestamp or data has changed
        // dataUpdatedAt ensures this effect runs on every fetch
        const next = [...prev, newPoint]
        if (next.length > 30) return next.slice(1)
        return next
      })
    }
  }, [stats, dataUpdatedAt])

  if (isLoading && history.length === 0) {
    return <div className="flex items-center justify-center h-64">Loading stats...</div>
  }

  const chartConfig: ChartConfig = {}
  const chartLines: { key: string; color: string; label: string; dashed?: boolean }[] = []
  
  const colors = [
    'var(--chart-1)',
    'var(--chart-2)',
    'var(--chart-3)',
    'var(--chart-4)',
    'var(--chart-5)',
  ]

  let colorIdx = 0
  stats?.forEach(stream => {
    const streamColor = colors[colorIdx % colors.length]
    
    // Line for total queued in stream
    const totalKey = `${stream.stream_name}_total`
    chartConfig[totalKey] = {
      label: `${stream.stream_name} (Total Queued)`,
      color: streamColor,
    }
    chartLines.push({ key: totalKey, color: streamColor, label: `${stream.stream_name} (Total Queued)` })
    
    stream.consumers.forEach(consumer => {
      const pendingKey = `${stream.stream_name}_${consumer.name}_pending`
      chartConfig[pendingKey] = {
        label: `${stream.stream_name} / ${consumer.name} (Pending)`,
        color: streamColor,
      }
      // Use dashed line for pending to distinguish from total
      chartLines.push({ key: pendingKey, color: streamColor, label: `${stream.stream_name} / ${consumer.name} (Pending)`, dashed: true })
    })
    
    colorIdx++
  })

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Worker Dashboard</h1>
        <p className="text-muted-foreground">
          Real-time monitoring of JetStream tasks and worker status.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {stats?.map(stream => 
          stream.consumers.map(consumer => (
            <Card key={`${stream.stream_name}-${consumer.name}`}>
              <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
                <CardTitle className="text-sm font-medium">
                  {stream.stream_name} / {consumer.name}
                </CardTitle>
                <Activity className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="flex justify-between items-end">
                  <div>
                    <div className="text-2xl font-bold">{consumer.pending}</div>
                    <p className="text-xs text-muted-foreground">Pending tasks</p>
                  </div>
                  <div className="text-right">
                    <div className="text-xl font-semibold text-primary">{consumer.ack_pending}</div>
                    <p className="text-xs text-muted-foreground">In flight</p>
                  </div>
                </div>
                {consumer.redelivered > 0 && (
                  <div className="mt-2 text-xs text-destructive flex items-center gap-1">
                    <PlayCircle className="w-3 h-3" />
                    {consumer.redelivered} redelivered
                  </div>
                )}
              </CardContent>
            </Card>
          ))
        )}
      </div>

      <Card className="col-span-4">
        <CardHeader>
          <CardTitle>Queue History</CardTitle>
          <CardDescription>
            Number of pending messages across all streams over the last 2.5 minutes.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ChartContainer config={chartConfig} className="aspect-auto h-[350px] w-full">
            <LineChart data={history} margin={{ top: 20, right: 30, left: 10, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" vertical={false} />
              <XAxis 
                dataKey="time" 
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                minTickGap={32}
              />
              <YAxis 
                tickLine={false}
                axisLine={false}
                tickMargin={8}
              />
              <ChartTooltip content={<ChartTooltipContent />} />
              <ChartLegend content={<ChartLegendContent />} />
              {chartLines.map(line => (
                <Line
                  key={line.key}
                  type="monotone"
                  dataKey={line.key}
                  stroke={line.color}
                  strokeDasharray={line.dashed ? "5 5" : "0"}
                  name={line.label}
                  dot={false}
                  strokeWidth={2}
                  animationDuration={300}
                />
              ))}
            </LineChart>
          </ChartContainer>
        </CardContent>
      </Card>
    </div>
  )
}
